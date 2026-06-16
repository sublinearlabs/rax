use std::{
    collections::{hash_map::Entry, HashMap},
    rc::Rc,
};

use dynasmrt::{dynasm, DynasmApi};

use crate::aot::{
    register_mapping::{MapTarget, XmmLane},
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::{AllocatedTemp, TempAllocator},
    translator::Translator,
};

/// Canonical location for a value currently usable as a GPR source.
///
/// Values may already reside in a mapped x86 register or be materialized into
/// a temporary register managed by `TempAllocator`.
#[derive(Clone)]
pub(super) enum ValueLoc<'a> {
    ConstZero,
    Mapped(X86Gpr),
    Temp(Rc<AllocatedTemp<'a>>),
}

impl<'a> ValueLoc<'a> {
    /// Returns the x86 GPR backing this value location.
    ///
    /// # Panics
    ///
    /// Panics for `ConstZero`, which has no backing x86 GPR.
    /// Zero-valued sources must be handled by lowering logic before requesting
    /// a concrete carrier register.
    fn gpr(&self) -> X86Gpr {
        match self {
            Self::ConstZero => panic!("should not materialize const a zero input"),
            Self::Mapped(reg) => *reg,
            Self::Temp(reg) => **reg.as_ref(),
        }
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) backing this value location.
    ///
    /// This is the source carrier register code used by instruction encoders.
    /// It is not a RISC-V register index.
    ///
    /// # Panics
    ///
    /// Panics for `ConstZero`, which has no backing x86 GPR.
    /// Zero-valued sources must be handled by lowering logic before requesting
    /// a concrete carrier id.
    fn id(&self) -> u8 {
        self.gpr().id()
    }
}

/// Prepared input ready for instruction emission.
///
/// Inputs may be materialized from mapped GPRs/XMM lanes or represented as
/// `ConstZero` when the architectural source is `x0`.
#[derive(Clone)]
pub(super) struct PreparedInput<'a> {
    pub(super) src: ValueLoc<'a>,
}

impl<'a> PreparedInput<'a> {
    /// Returns whether this prepared input represents architectural zero (`x0`).
    ///
    /// Callers should use this guard before requesting a concrete carrier via
    /// `gpr()`/`id()`.
    pub(super) fn is_zero(&self) -> bool {
        matches!(self.src, ValueLoc::ConstZero)
    }

    /// Returns the x86 GPR to use as the emitted instruction source.
    ///
    /// # Panics
    ///
    /// Panics when this input is `ConstZero`.
    /// Callers must branch on zero-valued inputs before calling this method.
    pub(super) fn gpr(&self) -> X86Gpr {
        self.src.gpr()
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared input.
    ///
    /// This is the source carrier register code used by instruction encoders.
    /// It is not a RISC-V register index.
    ///
    /// # Panics
    ///
    /// Panics when this input is `ConstZero`.
    /// Callers must branch on zero-valued inputs before calling this method.
    pub(super) fn id(&self) -> u8 {
        self.src.id()
    }
}

/// Prepared architectural destination bound to a computed source value.
///
/// A prepared output must be explicitly completed; dropping one without calling
/// `write_back`, `discard_zero_output`, or `commit_unchanged` is considered a
/// programmer error and panics.
pub(super) struct PreparedOutput<'a> {
    src: ValueLoc<'a>,
    dest: MapTarget,
    written_back: bool,
}

impl<'a> PreparedOutput<'a> {
    pub(super) fn new(src: ValueLoc<'a>, dest: MapTarget) -> Self {
        Self {
            src,
            dest,
            written_back: false,
        }
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared output
    /// source carrier.
    ///
    /// This is the source register code used by instruction encoders. It is
    /// not a destination map id and not a RISC-V register index.
    ///
    /// # Panics
    ///
    /// Panics when this output is `ConstZero`.
    pub(super) fn id(&self) -> u8 {
        // panic if zero output
        if self.is_zero() {
            panic!("PreparedOutput::id called on zero/elided output");
        }

        self.src.id()
    }

    /// Returns whether this prepared output targets architectural zero (`x0`)
    ///
    /// A zero output represents an elided architectural write destination and
    /// does not permit carrier-id lookup or write-back emission.
    pub(super) fn is_zero(&self) -> bool {
        matches!(self.dest, MapTarget::ConstZero)
    }

    /// Marks a zero output as intentionally discarded.
    ///
    /// This is the explicit completion path for `rd == x0`. Non-zero outputs
    /// must use `write_back()` instead.
    fn discard_zero(mut self) {
        if !self.is_zero() {
            self.written_back = true;
            panic!("PreparedOutput::discard_zero called for non-zero destination");
        }

        self.written_back = true;
    }

    /// Marks a non-zero output as intentionally unchanged.
    ///
    /// This is the explicit completion path for instructions that leave their
    /// destination value unchanged. Zero outputs must use `discard_zero()`.
    fn commit_unchanged(mut self) {
        if self.is_zero() {
            self.written_back = true;
            panic!("PreparedOutput::commit_unchanged called for ConstZero destination");
        }

        self.written_back = true;
    }

    /// Writes a computed source value back to its architectural destination.
    ///
    /// # Contract
    ///
    /// Must be called exactly once for each prepared output.
    ///
    /// Destination semantics are determined by `MapTarget`:
    /// - `ConstZero`: panics; use `InstructionContext::discard_zero_output()` instead
    /// - `Gpr`: source is written to mapped x86 GPR
    /// - `XmmShared`: source is written to selected shared XMM lane
    /// - `XmmExclusive`: source is written to exclusive XMM destination
    pub(super) fn write_back(mut self, translator: &mut Translator) {
        match self.dest {
            MapTarget::ConstZero => {
                self.written_back = true;
                panic!("PreparedOutput::write_back called for ConstZero destination")
            }
            MapTarget::Gpr(dst) => {
                if self.src.gpr() != dst {
                    dynasm!(translator.emitter
                        ; mov Rq(dst.id()), Rq(self.id())
                    );
                }
            }
            MapTarget::XmmExclusive(reg) => {
                dynasm!(translator.emitter
                    ; movq Rx(reg.id()), Rq(self.id())
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::Low,
            } => {
                // Use PINSRQ for shared-low writes to preserve the high 64-bit lane.
                // MOVQ xmm, r64 would clobber/zero the other lane and corrupt its paired shared value.
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(self.id()), 0
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::High,
            } => {
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(self.id()), 1
                );
            }
        }
        self.written_back = true;
    }

    fn suppress_drop_panic(mut self) {
        self.written_back = true;
    }
}

impl<'a> Drop for PreparedOutput<'a> {
    /// Enforces strict explicit completion before output teardown.
    fn drop(&mut self) {
        if !self.written_back {
            panic!(
                "PreparedOutput dropped before write_back, discard_zero_output, or commit_unchanged"
            );
        }
    }
}

/// Completion guard for contexts that intentionally have no architectural output.
///
/// A no-output context must still be explicitly completed so clobber restores are
/// emitted and missing termination remains visible as a lowering bug.
struct NoOutputCompletion {
    completed: bool,
}

impl NoOutputCompletion {
    fn new() -> Self {
        Self { completed: false }
    }

    fn complete(mut self) {
        self.completed = true;
    }

    fn suppress_drop_panic(mut self) {
        self.completed = true;
    }
}

impl Drop for NoOutputCompletion {
    fn drop(&mut self) {
        if !self.completed {
            panic!("No-output InstructionContext dropped before complete_no_output");
        }
    }
}

/// Builder for preparing instruction operands before AOT emission.
///
/// The builder collects architectural inputs, an optional architectural output,
/// and any x86 GPRs that the emitted instruction may clobber. `build()`
/// materializes the requested operands and emits any setup moves needed to
/// preserve clobbered mapped values.
pub(super) struct InstructionContextBuilder<const NI: usize, const NCT: usize> {
    /// Architectural source registers consumed by the instruction.
    inputs: Option<[RiscvRegister; NI]>,
    /// Architectural destination register produced by the instruction, if any.
    output: Option<RiscvRegister>,
    /// x86 GPRs that must be preserved across instruction emission.
    clobber_targets: Option<[X86Gpr; NCT]>,
}

impl<const NI: usize, const NCT: usize> InstructionContextBuilder<NI, NCT> {
    /// Creates an empty instruction context builder.
    ///
    /// Inputs, output, and clobber targets may be supplied with the builder
    /// methods before calling `build()`.
    pub(super) fn new() -> Self {
        Self {
            inputs: None,
            output: None,
            clobber_targets: None,
        }
    }

    /// Sets the architectural source registers for this instruction.
    pub(super) fn set_inputs(mut self, inputs: [RiscvRegister; NI]) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Sets the architectural destination register for this instruction.
    pub(super) fn set_output(mut self, output: RiscvRegister) -> Self {
        self.output = Some(output);
        self
    }

    /// Marks x86 GPRs that must not be clobbered by instruction emission.
    ///
    /// During `build()`, any live mapped value in these registers is moved to a
    /// temp and a restore output is recorded in the resulting
    /// `InstructionContext`.
    pub(super) fn ensure_no_clobber(mut self, clobber_targets: [X86Gpr; NCT]) -> Self {
        self.clobber_targets = Some(clobber_targets);
        self
    }

    /// Builds a prepared instruction context and emits required setup moves.
    ///
    /// Materializes XMM-backed inputs into temp GPRs, preserves requested
    /// clobber targets, reuses materialized values through an internal cache,
    /// and prepares the architectural output for later write-back when present.
    ///
    /// # Panics
    ///
    /// Panics when a required temp GPR cannot be allocated.
    /// Zero outputs must be completed with
    /// `InstructionContext::discard_zero_output()`.
    pub(super) fn build<'a>(
        self,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> InstructionContext<'a, NI, NCT> {
        let inputs = match self.inputs {
            Some(inputs) => inputs,
            None => {
                assert_eq!(NI, 0, "inputs must be present");
                std::array::from_fn(|_| unreachable!("zero-input context has no elements"))
            }
        };
        let output = self.output;
        let mut cache: HashMap<MapTarget, ValueLoc<'a>> = HashMap::new();

        let (clobber_restore, reserved_temps) =
            Self::preserve_clobbers(self.clobber_targets, &mut cache, translator, temp_allocator);
        let prepared_inputs = Self::prepare_inputs(inputs, &mut cache, translator, temp_allocator);
        let prepared_output = output
            .map(|output| Self::prepare_output(output, &mut cache, translator, temp_allocator));
        let no_output_completion = prepared_output.is_none().then(NoOutputCompletion::new);

        InstructionContext {
            inputs: prepared_inputs.try_into().unwrap_or_else(|_| {
                unreachable!(
                    "we push one prepared_input for each input, hence should be the same size"
                )
            }),
            output: prepared_output,
            no_output_completion,
            clobber_restore,
            reserved_temps,
        }
    }

    /// Preserves mapped GPR values and reserves temp GPRs that emission may clobber.
    ///
    /// Temp clobbers are reserved without save/restore because they do not hold
    /// architectural state. Mapped clobbers are copied into a temp once and
    /// cached so later input/output preparation reuses the relocated carrier.
    /// The returned outputs restore those values during context completion.
    fn preserve_clobbers<'a>(
        clobber_targets: Option<[X86Gpr; NCT]>,
        cache: &mut HashMap<MapTarget, ValueLoc<'a>>,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> (Vec<PreparedOutput<'a>>, Vec<AllocatedTemp<'a>>) {
        let mut clobber_restore = vec![];
        let mut reserved_temps = vec![];

        let Some(clobber_targets) = clobber_targets else {
            return (clobber_restore, reserved_temps);
        };

        let mut unique_targets = Vec::with_capacity(NCT);
        for clobbered_reg in clobber_targets {
            if !unique_targets.contains(&clobbered_reg) {
                unique_targets.push(clobbered_reg);
            }
        }

        for clobbered_reg in &unique_targets {
            if !temp_allocator.is_temp(clobbered_reg) {
                continue;
            }

            let reserved = temp_allocator
                .allocate_specific(*clobbered_reg)
                .unwrap_or_else(|_| panic!("instruction context could not reserve temp GPR"));
            reserved_temps.push(reserved);
        }

        for clobbered_reg in unique_targets {
            if temp_allocator.is_temp(&clobbered_reg) {
                continue;
            }

            let target = MapTarget::Gpr(clobbered_reg);
            let Entry::Vacant(entry) = cache.entry(target) else {
                continue;
            };

            let temp_reg = Self::alloc_temp(temp_allocator);
            dynasm!(translator.emitter ; mov Rq(temp_reg.id()), Rq(clobbered_reg.id()));
            entry.insert(temp_reg.clone());
            clobber_restore.push(PreparedOutput::new(temp_reg, target));
        }

        (clobber_restore, reserved_temps)
    }

    /// Prepares architectural inputs in source-register order.
    ///
    /// Reuses cached carriers for duplicate inputs or values already relocated
    /// by clobber preservation, avoiding repeated materialization.
    fn prepare_inputs<'a>(
        inputs: [RiscvRegister; NI],
        cache: &mut HashMap<MapTarget, ValueLoc<'a>>,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> Vec<PreparedInput<'a>> {
        let mut prepared_inputs = Vec::with_capacity(NI);

        for input in inputs {
            let target = *translator.reg_map.get(&input);
            let src = Self::prepare_input_target(target, cache, translator, temp_allocator);
            prepared_inputs.push(PreparedInput { src });
        }

        prepared_inputs
    }

    /// Returns a GPR-usable carrier for one mapping target.
    ///
    /// GPR targets are used directly, `x0` remains a constant-zero sentinel,
    /// and XMM-backed values are extracted into temps. Materialized targets are
    /// cached so aliases and duplicates share the same carrier.
    fn prepare_input_target<'a>(
        target: MapTarget,
        cache: &mut HashMap<MapTarget, ValueLoc<'a>>,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> ValueLoc<'a> {
        match cache.entry(target) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => match target {
                MapTarget::ConstZero => ValueLoc::ConstZero,
                MapTarget::Gpr(x86_gpr) => ValueLoc::Mapped(x86_gpr),
                MapTarget::XmmExclusive(reg)
                | MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::Low,
                } => {
                    let val = Self::alloc_temp(temp_allocator);
                    dynasm!(translator.emitter ; movq Rq(val.id()), Rx(reg.id()));
                    entry.insert(val.clone());
                    val
                }
                MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::High,
                } => {
                    let val = Self::alloc_temp(temp_allocator);
                    dynasm!(translator.emitter ; pextrq Rq(val.id()), Rx(reg.id()), 1);
                    entry.insert(val.clone());
                    val
                }
            },
        }
    }

    /// Prepares the architectural output carrier for instruction emission.
    ///
    /// If the output target was already materialized as an input or relocated
    /// clobber, the existing carrier is reused. XMM destinations receive a temp
    /// carrier and are written back by `InstructionContext::write_back()`.
    fn prepare_output<'a>(
        output: RiscvRegister,
        cache: &mut HashMap<MapTarget, ValueLoc<'a>>,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> PreparedOutput<'a> {
        let target = *translator.reg_map.get(&output);

        match cache.entry(target) {
            Entry::Occupied(entry) => PreparedOutput::new(entry.get().clone(), target),
            Entry::Vacant(entry) => {
                let src = match target {
                    MapTarget::ConstZero => ValueLoc::ConstZero,
                    MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
                    MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                        Self::alloc_temp(temp_allocator)
                    }
                };
                entry.insert(src.clone());
                PreparedOutput::new(src, target)
            }
        }
    }

    /// Allocates a temp GPR wrapped as a `ValueLoc`.
    ///
    /// Centralizes the allocation panic so all temp-pressure failures report the
    /// same instruction-context error.
    fn alloc_temp<'a>(temp_allocator: &'a TempAllocator) -> ValueLoc<'a> {
        let temp = temp_allocator
            .allocate()
            .unwrap_or_else(|_| panic!("instruction context could not allocate temp GPR"));
        ValueLoc::Temp(Rc::new(temp))
    }
}

/// Prepared operands and deferred write-back state for one instruction.
///
/// An instruction context owns the prepared inputs, the prepared architectural
/// output when present, and any restore operations required for protected
/// clobber targets. Instruction implementations should use `inputs()` and
/// `output()` while emitting machine code, then finish by calling `write_back()`,
/// `discard_zero_output()`, `commit_unchanged()`, or `complete_no_output()`.
///
/// # Contract
///
/// Output contexts must be completed with `write_back()`,
/// `discard_zero_output()`, or `commit_unchanged()`. No-output contexts must be
/// completed with `complete_no_output()`.
pub(super) struct InstructionContext<'a, const NI: usize, const NCT: usize> {
    /// Prepared source operands available for instruction emission.
    inputs: [PreparedInput<'a>; NI],
    /// Prepared destination carrier for the instruction result.
    output: Option<PreparedOutput<'a>>,
    /// Completion guard for instructions with no architectural output.
    no_output_completion: Option<NoOutputCompletion>,
    /// Deferred restores for mapped values moved out of clobber targets.
    clobber_restore: Vec<PreparedOutput<'a>>,
    /// Reserved temp clobber targets kept unavailable for this context lifetime.
    reserved_temps: Vec<AllocatedTemp<'a>>,
}

impl<'a, const NI: usize, const NCT: usize> InstructionContext<'a, NI, NCT> {
    /// Returns the prepared inputs for this instruction.
    ///
    /// The returned inputs are ordered to match the architectural source
    /// registers supplied to `InstructionContextBuilder::set_inputs()`.
    pub(super) fn inputs(&self) -> &[PreparedInput<'a>; NI] {
        &self.inputs
    }

    /// Returns the prepared architectural output.
    ///
    /// The output may be queried for its carrier id during instruction
    /// emission. Complete the context with `write_back()` for computed
    /// destinations, `commit_unchanged()` for unchanged destinations, or
    /// `discard_zero_output()` for `rd == x0`.
    pub(super) fn output(&self) -> &PreparedOutput<'a> {
        self.output
            .as_ref()
            .expect("InstructionContext::output called for no-output context")
    }

    /// Writes the prepared output and any clobber restores back to their mapped
    /// architectural locations.
    ///
    /// Use this completion path only for non-zero architectural destinations.
    ///
    /// # Panics
    ///
    /// Panics if the output is `ConstZero`; use `discard_zero_output()` for
    /// `rd == x0`.
    pub(super) fn write_back(self, translator: &mut Translator) {
        let InstructionContext {
            output,
            no_output_completion,
            clobber_restore,
            ..
        } = self;

        let Some(output) = output else {
            if let Some(no_output_completion) = no_output_completion {
                no_output_completion.suppress_drop_panic();
            }
            Self::discard_restores(clobber_restore);
            panic!("InstructionContext::write_back called for no-output context");
        };

        if output.is_zero() {
            // Suppress drop guards so the API misuse panic below is the only
            // panic during unwinding.
            Self::discard_restores(clobber_restore);
            output.suppress_drop_panic();
            panic!("InstructionContext::write_back called for ConstZero output");
        }

        output.write_back(translator);
        Self::write_restores(clobber_restore, translator);
    }

    /// Explicitly discards an architectural zero (`x0`) output.
    ///
    /// This is the required completion path for instructions whose destination
    /// is `rd == x0`. It emits no destination write, but still restores any
    /// clobber-preserved mapped values.
    ///
    /// # Panics
    ///
    /// Panics if the output is not `ConstZero`; use `write_back()` for real
    /// destinations.
    pub(super) fn discard_zero_output(self, translator: &mut Translator) {
        let InstructionContext {
            output,
            no_output_completion,
            clobber_restore,
            ..
        } = self;

        let Some(output) = output else {
            if let Some(no_output_completion) = no_output_completion {
                no_output_completion.suppress_drop_panic();
            }
            Self::discard_restores(clobber_restore);
            panic!("InstructionContext::discard_zero_output called for no-output context");
        };

        if !output.is_zero() {
            // Suppress drop guards so the API misuse panic below is the only
            // panic during unwinding.
            Self::discard_restores(clobber_restore);
            output.suppress_drop_panic();
            panic!("InstructionContext::discard_zero_output called for non-zero output");
        }

        output.discard_zero();
        Self::write_restores(clobber_restore, translator);
    }

    /// Explicitly commits an unchanged non-zero architectural output.
    ///
    /// This completion path emits no destination write, but still restores any
    /// clobber-preserved mapped values.
    ///
    /// # Panics
    ///
    /// Panics if the output is `ConstZero`; use `discard_zero_output()` for
    /// `rd == x0`.
    pub(super) fn commit_unchanged(self, translator: &mut Translator) {
        let InstructionContext {
            output,
            no_output_completion,
            clobber_restore,
            ..
        } = self;

        let Some(output) = output else {
            if let Some(no_output_completion) = no_output_completion {
                no_output_completion.suppress_drop_panic();
            }
            Self::discard_restores(clobber_restore);
            panic!("InstructionContext::commit_unchanged called for no-output context");
        };

        if output.is_zero() {
            // Suppress drop guards so the API misuse panic below is the only
            // panic during unwinding.
            Self::discard_restores(clobber_restore);
            output.suppress_drop_panic();
            panic!("InstructionContext::commit_unchanged called for ConstZero output");
        }

        output.commit_unchanged();
        Self::write_restores(clobber_restore, translator);
    }

    /// Explicitly completes a context with no architectural output.
    ///
    /// This completion path emits no destination write, but still restores any
    /// clobber-preserved mapped values.
    ///
    /// # Panics
    ///
    /// Panics if the context has an output; use one of the output completion
    /// paths instead.
    pub(super) fn complete_no_output(self, translator: &mut Translator) {
        let InstructionContext {
            output,
            no_output_completion,
            clobber_restore,
            ..
        } = self;

        if let Some(output) = output {
            Self::discard_restores(clobber_restore);
            output.suppress_drop_panic();
            panic!("InstructionContext::complete_no_output called for output context");
        }

        let no_output_completion = no_output_completion
            .expect("InstructionContext::complete_no_output missing no-output guard");
        no_output_completion.complete();
        Self::write_restores(clobber_restore, translator);
    }

    /// Writes clobber-preserved values back to their mapped GPRs.
    fn write_restores(clobber_restore: Vec<PreparedOutput<'a>>, translator: &mut Translator) {
        for restore in clobber_restore {
            restore.write_back(translator);
        }
    }

    /// Marks clobber restores complete before an intentional context-level panic.
    fn discard_restores(clobber_restore: Vec<PreparedOutput<'a>>) {
        for restore in clobber_restore {
            restore.suppress_drop_panic();
        }
    }
}

#[cfg(test)]
mod tests {
    use dynasmrt::x64::Assembler;

    use crate::aot::{register_mapping::RegisterMapping, registers::X86Xmm};

    use super::*;

    fn new_translator() -> Translator {
        Translator::new(
            Assembler::new().unwrap(),
            RegisterMapping::default_plan(),
            0,
        )
    }

    fn new_translator_all_xmm_shared() -> Translator {
        let mut builder = RegisterMapping::builder();
        for idx in 1..32 {
            let reg = RiscvRegister::from_index(idx).expect("valid riscv reg index");
            let lane_idx = idx - 1;
            let xmm = X86Xmm::from_index(lane_idx / 2).expect("valid xmm index");
            let lane = if lane_idx % 2 == 0 {
                XmmLane::Low
            } else {
                XmmLane::High
            };
            builder
                .map_xmm_shared(reg, xmm, lane)
                .expect("builder assignment should succeed");
        }

        Translator::new(
            Assembler::new().unwrap(),
            builder
                .build()
                .expect("builder should produce valid mapping"),
            0,
        )
    }

    #[test]
    fn gpr_context_does_not_require_temp() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A1])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs()[0].id(), X86Gpr::Rsi.id());
        assert_eq!(ctx.output().id(), X86Gpr::Rdi.id());
        ctx.write_back(&mut translator);
    }

    #[test]
    fn duplicate_xmm_input_materializes_once() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::<2, 0>::new()
            .set_inputs([RiscvRegister::S0, RiscvRegister::S0])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        let (first, second) = match (&ctx.inputs()[0].src, &ctx.inputs()[1].src) {
            (ValueLoc::Temp(first), ValueLoc::Temp(second)) => (first, second),
            _ => panic!("expected duplicate XMM inputs to share a temp carrier"),
        };

        assert!(Rc::ptr_eq(first, second));
        assert_eq!(ctx.inputs()[0].id(), X86Gpr::R11.id());
        ctx.write_back(&mut translator);
    }

    #[test]
    fn xmm_input_and_output_same_target_reuse_materialized_carrier() {
        let mut translator = new_translator_all_xmm_shared();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs()[0].id(), ctx.output().id());
        ctx.write_back(&mut translator);
    }

    #[test]
    fn clobbered_gpr_input_is_relocated_once() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::A1)
            .ensure_no_clobber([X86Gpr::Rdi])
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs()[0].id(), X86Gpr::R11.id());
        assert_ne!(ctx.inputs()[0].id(), X86Gpr::Rdi.id());
        assert_eq!(ctx.clobber_restore.len(), 1);
        ctx.write_back(&mut translator);

        assert_eq!(
            translator.finalize(),
            vec![0x49, 0x89, 0xfb, 0x4c, 0x89, 0xdf]
        );
    }

    #[test]
    fn duplicate_clobber_target_is_preserved_once() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([])
            .set_output(RiscvRegister::A1)
            .ensure_no_clobber([X86Gpr::Rdi, X86Gpr::Rdi])
            .build(&mut translator, &temps);

        assert_eq!(ctx.clobber_restore.len(), 1);
        ctx.write_back(&mut translator);
    }

    #[test]
    fn temp_clobber_reserves_without_restore() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::<0, 1>::new()
            .set_inputs([])
            .ensure_no_clobber([X86Gpr::R11])
            .build(&mut translator, &temps);

        assert_eq!(ctx.clobber_restore.len(), 0);
        assert_eq!(ctx.reserved_temps.len(), 1);
        ctx.complete_no_output(&mut translator);

        assert_eq!(translator.finalize(), Vec::<u8>::new());
    }

    #[test]
    fn temp_clobber_blocks_later_temp_allocation() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11, X86Gpr::R12]);

        let ctx = InstructionContextBuilder::<1, 1>::new()
            .set_inputs([RiscvRegister::S0])
            .ensure_no_clobber([X86Gpr::R11])
            .build(&mut translator, &temps);

        assert_eq!(ctx.clobber_restore.len(), 0);
        assert_eq!(ctx.reserved_temps.len(), 1);
        assert_eq!(ctx.inputs()[0].id(), X86Gpr::R12.id());
        ctx.complete_no_output(&mut translator);
    }

    #[test]
    fn temp_clobbers_are_reserved_before_mapped_clobbers() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11, X86Gpr::R12]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::A1)
            .ensure_no_clobber([X86Gpr::Rdi, X86Gpr::R11])
            .build(&mut translator, &temps);

        assert_eq!(ctx.reserved_temps.len(), 1);
        assert_eq!(ctx.clobber_restore.len(), 1);
        assert_eq!(ctx.inputs()[0].id(), X86Gpr::R12.id());
        assert_ne!(ctx.inputs()[0].id(), X86Gpr::R11.id());

        ctx.write_back(&mut translator);
    }

    #[test]
    fn duplicate_temp_clobber_reserves_once() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::<0, 2>::new()
            .set_inputs([])
            .ensure_no_clobber([X86Gpr::R11, X86Gpr::R11])
            .build(&mut translator, &temps);

        assert_eq!(ctx.clobber_restore.len(), 0);
        assert_eq!(ctx.reserved_temps.len(), 1);
        ctx.complete_no_output(&mut translator);
    }

    #[test]
    #[should_panic]
    fn xmm_input_panics_when_temp_is_required_but_unavailable() {
        let mut translator = new_translator_all_xmm_shared();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::Ra)
            .build(&mut translator, &temps);
    }

    #[test]
    fn no_output_context_builds_and_completes_explicitly() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A0])
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs()[0].id(), X86Gpr::Rdi.id());
        ctx.complete_no_output(&mut translator);

        assert_eq!(translator.finalize(), Vec::<u8>::new());
    }

    #[test]
    fn output_panics_for_no_output_context() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A0])
            .build(&mut translator, &temps);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = ctx.output();
        }));

        assert!(result.is_err());
        ctx.complete_no_output(&mut translator);
    }

    #[test]
    fn complete_no_output_restores_clobbers() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([RiscvRegister::A0])
            .ensure_no_clobber([X86Gpr::Rdi])
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs()[0].id(), X86Gpr::R11.id());
        ctx.complete_no_output(&mut translator);

        assert_eq!(
            translator.finalize(),
            vec![0x49, 0x89, 0xfb, 0x4c, 0x89, 0xdf]
        );
    }

    #[test]
    #[should_panic(expected = "No-output InstructionContext dropped before complete_no_output")]
    fn no_output_context_drop_without_complete_panics() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .build(&mut translator, &temps);
    }

    #[test]
    #[should_panic(expected = "InstructionContext::complete_no_output called for output context")]
    fn complete_no_output_panics_for_output_context() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        ctx.complete_no_output(&mut translator);
    }

    #[test]
    #[should_panic(expected = "InstructionContext::write_back called for no-output context")]
    fn write_back_panics_for_no_output_context() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .build(&mut translator, &temps);

        ctx.write_back(&mut translator);
    }

    #[test]
    #[should_panic(
        expected = "InstructionContext::discard_zero_output called for no-output context"
    )]
    fn discard_zero_output_panics_for_no_output_context() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .build(&mut translator, &temps);

        ctx.discard_zero_output(&mut translator);
    }

    #[test]
    #[should_panic(expected = "InstructionContext::commit_unchanged called for no-output context")]
    fn commit_unchanged_panics_for_no_output_context() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .build(&mut translator, &temps);

        ctx.commit_unchanged(&mut translator);
    }

    #[test]
    #[should_panic(expected = "inputs must be present")]
    fn non_zero_input_context_panics_when_inputs_are_missing() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<1, 0>::new()
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);
    }

    #[test]
    fn zero_input_context_defaults_missing_inputs_to_empty() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        assert_eq!(ctx.inputs().len(), 0);
        ctx.commit_unchanged(&mut translator);
    }

    #[test]
    fn zero_output_context_builds_and_discards_explicitly() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::Zero)
            .build(&mut translator, &temps);

        assert!(ctx.output().is_zero());
        ctx.discard_zero_output(&mut translator);
    }

    #[test]
    #[should_panic(expected = "InstructionContext::write_back called for ConstZero output")]
    fn write_back_panics_for_zero_output() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::Zero)
            .build(&mut translator, &temps);

        ctx.write_back(&mut translator);
    }

    #[test]
    #[should_panic(expected = "InstructionContext::discard_zero_output called for non-zero output")]
    fn discard_zero_output_panics_for_non_zero_output() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        ctx.discard_zero_output(&mut translator);
    }

    #[test]
    fn discard_zero_output_restores_clobbers() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::Zero)
            .ensure_no_clobber([X86Gpr::Rdi])
            .build(&mut translator, &temps);

        ctx.discard_zero_output(&mut translator);

        assert_eq!(
            translator.finalize(),
            vec![0x49, 0x89, 0xfb, 0x4c, 0x89, 0xdf]
        );
    }

    #[test]
    fn commit_unchanged_non_zero_output_emits_no_write() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);

        ctx.commit_unchanged(&mut translator);

        assert_eq!(translator.finalize(), Vec::<u8>::new());
    }

    #[test]
    fn commit_unchanged_restores_clobbers() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![X86Gpr::R11]);

        let ctx = InstructionContextBuilder::new()
            .set_inputs([RiscvRegister::A0])
            .set_output(RiscvRegister::A1)
            .ensure_no_clobber([X86Gpr::Rdi])
            .build(&mut translator, &temps);

        ctx.commit_unchanged(&mut translator);

        assert_eq!(
            translator.finalize(),
            vec![0x49, 0x89, 0xfb, 0x4c, 0x89, 0xdf]
        );
    }

    #[test]
    #[should_panic(expected = "InstructionContext::commit_unchanged called for ConstZero output")]
    fn commit_unchanged_panics_for_zero_output() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let ctx = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::Zero)
            .build(&mut translator, &temps);

        ctx.commit_unchanged(&mut translator);
    }

    #[test]
    #[should_panic(
        expected = "PreparedOutput dropped before write_back, discard_zero_output, or commit_unchanged"
    )]
    fn zero_output_drop_without_discard_panics() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::Zero)
            .build(&mut translator, &temps);
    }

    #[test]
    #[should_panic(
        expected = "PreparedOutput dropped before write_back, discard_zero_output, or commit_unchanged"
    )]
    fn context_drop_without_write_back_panics() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);
    }
}
