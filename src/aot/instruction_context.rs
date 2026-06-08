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
/// A prepared output must be explicitly written back; dropping one without
/// calling `write_back` is considered a programmer error and panics.
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
    fn is_zero(&self) -> bool {
        matches!(self.dest, MapTarget::ConstZero)
    }

    /// Writes a computed source value back to its architectural destination.
    ///
    /// # Contract
    ///
    /// Must be called exactly once for each prepared output.
    ///
    /// Destination semantics are determined by `MapTarget`:
    /// - `ConstZero`: destination write is elided
    /// - `Gpr`: source is written to mapped x86 GPR
    /// - `XmmShared`: source is written to selected shared XMM lane
    /// - `XmmExclusive`: source is written to exclusive XMM destination
    pub(super) fn write_back(mut self, translator: &mut Translator) {
        let src = self.id();
        match self.dest {
            MapTarget::ConstZero => {
                unreachable!(
                    "write_back invariant violated: ConstZero destination should never reach PreparedOutput"
                )
            }
            MapTarget::Gpr(dst) => {
                if self.src.gpr() != dst {
                    dynasm!(translator.emitter
                        ; mov Rq(dst.id()), Rq(src)
                    );
                }
            }
            MapTarget::XmmExclusive(reg) => {
                dynasm!(translator.emitter
                    ; movq Rx(reg.id()), Rq(src)
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::Low,
            } => {
                // Use PINSRQ for shared-low writes to preserve the high 64-bit lane.
                // MOVQ xmm, r64 would clobber/zero the other lane and corrupt its paired shared value.
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(src), 0
                );
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::High,
            } => {
                dynasm!(translator.emitter
                    ; pinsrq Rx(reg.id()), Rq(src), 1
                );
            }
        }
        self.written_back = true;
    }
}

impl<'a> Drop for PreparedOutput<'a> {
    /// Enforces strict write-back completion before output teardown.
    fn drop(&mut self) {
        if !self.written_back {
            panic!("PreparedOutput dropped before write_back");
        }
    }
}

/// Builder for preparing instruction operands before AOT emission.
///
/// The builder collects architectural inputs, one architectural output, and any
/// x86 GPRs that the emitted instruction may clobber. `build()` materializes the
/// requested operands into `PreparedInput`/`PreparedOutput` values and emits any
/// setup moves needed to preserve clobbered mapped values.
struct InstructionContextBuilder<const NI: usize, const NCT: usize> {
    /// Architectural source registers consumed by the instruction.
    inputs: Option<[RiscvRegister; NI]>,
    /// Architectural destination register produced by the instruction.
    output: Option<RiscvRegister>,
    /// x86 GPRs that must be preserved across instruction emission.
    clobber_targets: Option<[X86Gpr; NCT]>,
}

impl<const NI: usize, const NCT: usize> InstructionContextBuilder<NI, NCT> {
    /// Creates an empty instruction context builder.
    ///
    /// Inputs, output, and clobber targets may be supplied with the builder
    /// methods before calling `build()`.
    fn new() -> Self {
        Self {
            inputs: None,
            output: None,
            clobber_targets: None,
        }
    }

    /// Sets the architectural source registers for this instruction.
    fn set_inputs(mut self, inputs: [RiscvRegister; NI]) -> Self {
        self.inputs = Some(inputs);
        self
    }

    /// Sets the architectural destination register for this instruction.
    fn set_output(mut self, output: RiscvRegister) -> Self {
        self.output = Some(output);
        self
    }

    /// Marks x86 GPRs that must not be clobbered by instruction emission.
    ///
    /// During `build()`, any live mapped value in these registers is moved to a
    /// temp and a restore output is recorded in the resulting
    /// `InstructionContext`.
    fn ensure_no_clobber(mut self, clobber_targets: [X86Gpr; NCT]) -> Self {
        self.clobber_targets = Some(clobber_targets);
        self
    }

    /// Builds a prepared instruction context and emits required setup moves.
    ///
    /// Materializes XMM-backed inputs into temp GPRs, preserves requested
    /// clobber targets, reuses materialized values through an internal cache,
    /// and prepares the architectural output for later write-back.
    ///
    /// # Panics
    ///
    /// Panics when a required temp GPR cannot be allocated.
    /// Panics when the output is missing.
    /// Panics when the output maps to `ConstZero`; lowering must handle
    /// `rd = x0` before building an instruction context.
    fn build<'a>(
        self,
        translator: &mut Translator,
        temp_allocator: &'a TempAllocator,
    ) -> InstructionContext<'a, NI, NCT> {
        let mut cache: HashMap<MapTarget, ValueLoc> = HashMap::new();

        let mut clobber_restore = vec![];
        let mut prepared_inputs = Vec::with_capacity(NI);
        let mut prepared_output = None;

        // handle clobber
        if let Some(clobber_targets) = self.clobber_targets {
            for clobbered_reg in clobber_targets {
                let target = MapTarget::Gpr(clobbered_reg);
                let Entry::Vacant(entry) = cache.entry(target) else {
                    continue;
                };

                let temp_reg = ValueLoc::Temp(Rc::new(temp_allocator.allocate().unwrap()));

                dynasm!(translator.emitter ; mov Rq(clobbered_reg.id()), Rq(temp_reg.id()));

                entry.insert(temp_reg.clone());

                clobber_restore.push(PreparedOutput::new(temp_reg, target));
            }
        }

        // handle input
        if let Some(inputs) = self.inputs {
            for input in inputs {
                let target = translator.reg_map.get(&input);

                let prepared_input = match cache.entry(*target) {
                    Entry::Occupied(entry) => PreparedInput {
                        src: entry.get().clone(),
                    },
                    Entry::Vacant(entry) => {
                        // here is where we actually handle the input creation
                        // we need to match the target and then produce some prepared input

                        match target {
                            MapTarget::ConstZero => PreparedInput {
                                src: ValueLoc::ConstZero,
                            },
                            MapTarget::Gpr(x86_gpr) => PreparedInput {
                                src: ValueLoc::Mapped(*x86_gpr),
                            },
                            MapTarget::XmmExclusive(reg)
                            | MapTarget::XmmShared {
                                reg,
                                lane: XmmLane::Low,
                            } => {
                                // TODO: handle unwrap here and else where
                                let temp = temp_allocator.allocate().unwrap();
                                dynasm!(translator.emitter ; movq Rq(temp.id()), Rx(reg.id()));
                                let val = ValueLoc::Temp(Rc::new(temp));
                                entry.insert(val.clone());
                                PreparedInput { src: val }
                            }
                            MapTarget::XmmShared { reg, lane } => {
                                let temp = temp_allocator.allocate().unwrap();
                                dynasm!(translator.emitter ; pextrq Rq(temp.id()), Rx(reg.id()), 1);
                                let val = ValueLoc::Temp(Rc::new(temp));
                                entry.insert(val.clone());
                                PreparedInput { src: val }
                            }
                        }
                    }
                };

                prepared_inputs.push(prepared_input);
            }
        }

        // handle output
        if let Some(output) = self.output {
            let target = translator.reg_map.get(&output);

            prepared_output = Some(match cache.entry(*target) {
                Entry::Occupied(entry) => PreparedOutput::new(entry.get().clone(), *target),
                Entry::Vacant(entry) => {
                    let src = match *target {
                        MapTarget::ConstZero => {
                            panic!("prepare_output invariant violated: x0/ConstZero destination must be handled by lowering before prepare_output")
                        }
                        MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
                        MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                            let temp = temp_allocator.allocate().unwrap_or_else(|_| {
                                panic!("prepare_output could not allocate temp GPR")
                            });
                            ValueLoc::Temp(Rc::new(temp))
                        }
                    };
                    entry.insert(src.clone());
                    PreparedOutput::new(src, *target)
                }
            });
        }

        // TODO: return the instruction context
        InstructionContext {
            inputs: prepared_inputs.try_into().unwrap_or_else(|_| {
                unreachable!(
                    "we push one prepared_input for each input, hence should be the same size"
                )
            }),
            output: prepared_output.expect("output must be present"),
            clobber_restore,
        }
    }
}

/// Prepared operands and deferred write-back state for one instruction.
///
/// An instruction context owns the prepared inputs, the prepared architectural
/// output, and any restore operations required for protected clobber targets.
/// Instruction implementations should use `inputs()` and `output()` while
/// emitting machine code, then finish by calling `write_back()`.
///
/// # Contract
///
/// `write_back()` must be called exactly once after instruction emission.
struct InstructionContext<'a, const NI: usize, const NCT: usize> {
    /// Prepared source operands available for instruction emission.
    inputs: [PreparedInput<'a>; NI],
    /// Prepared destination carrier for the instruction result.
    output: PreparedOutput<'a>,
    /// Deferred restores for mapped values moved out of clobber targets.
    clobber_restore: Vec<PreparedOutput<'a>>,
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
    /// emission. Architectural write-back is performed by `write_back()`.
    pub(super) fn output(&self) -> &PreparedOutput<'a> {
        &self.output
    }

    /// Writes the prepared output and any clobber restores back to their mapped
    /// architectural locations.
    ///
    /// # Contract
    ///
    /// Must be called exactly once after instruction emission.
    pub(super) fn write_back(self, translator: &mut Translator) {
        self.output.write_back(translator);

        for restore in self.clobber_restore {
            restore.write_back(translator);
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
    #[should_panic(expected = "output must be present")]
    fn build_panics_when_output_is_missing() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<1, 0>::new()
            .set_inputs([RiscvRegister::A0])
            .build(&mut translator, &temps);
    }

    #[test]
    #[should_panic(expected = "prepare_output invariant violated: x0/ConstZero")]
    fn build_panics_when_output_maps_to_constzero() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::Zero)
            .build(&mut translator, &temps);
    }

    #[test]
    #[should_panic(expected = "PreparedOutput dropped before write_back")]
    fn context_drop_without_write_back_panics() {
        let mut translator = new_translator();
        let temps = TempAllocator::new(vec![]);

        let _ = InstructionContextBuilder::<0, 0>::new()
            .set_inputs([])
            .set_output(RiscvRegister::A0)
            .build(&mut translator, &temps);
    }
}
