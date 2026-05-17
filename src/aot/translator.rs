use std::collections::HashMap;

use dynasmrt::{dynasm, x64::Assembler, AssemblyOffset, DynasmApi, DynamicLabel};

use crate::aot::{
    register_mapping::{MapTarget, MappingPlan, RegisterMapping, XmmLane},
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::{AllocatedTemp, TempAllocator},
};

/// AOT translator state used while lowering RISC-V instructions to x86.
///
/// This type owns the emitter and all translation-local state required to
/// materialize inputs and stage outputs for architectural write-back.
struct Translator {
    emitter: Assembler,
    reg_map: RegisterMapping,
    temp_allocator: TempAllocator,
    cf: ControlFlowState,
}

/// Control-flow translation state scoped to a translator instance.
struct ControlFlowState {
    /// RISC-V PC of the instruction currently being translated.
    ///
    /// This advances in translation order and is used for instruction-local
    /// control-flow semantics such as computing return PCs for jumps.
    current_riscv_pc: u64,
    /// Anchor PC for the translated RISC-V region.
    ///
    /// Slot 0 in the jump table corresponds to this PC.
    base_riscv_pc: u64,
    /// Mapping from known RISC-V target PCs to x86 dynamic labels.
    ///
    /// This supports direct branch/jump emission when the target is known at
    /// translation time.
    pc_labels: HashMap<u64, DynamicLabel>,
    /// Indexed x86 entry offsets used for runtime-indirect targets.
    ///
    /// Under the v1 non-compressed assumption, each slot represents one
    /// 4-byte RISC-V instruction position relative to `base_riscv_pc`.
    jump_table: Vec<AssemblyOffset>,
    /// Dynamic label that marks the start of the emitted jump-table data.
    ///
    /// Indirect jump paths use this as the base for indexed table lookups.
    jt_label: DynamicLabel,
}

/// Canonical location for a value currently usable as a GPR source.
///
/// Values may already reside in a mapped x86 register or be materialized into
/// a temporary register managed by `TempAllocator`.
enum ValueLoc<'a> {
    Mapped(X86Gpr),
    Temp(AllocatedTemp<'a>),
}

impl<'a> ValueLoc<'a> {
    /// Returns the x86 GPR backing this value location.
    fn gpr(&self) -> X86Gpr {
        match self {
            Self::Mapped(reg) => *reg,
            Self::Temp(reg) => **reg,
        }
    }
}

/// Prepared non-zero input ready for instruction emission.
///
/// The translator's strict input policy requires callers to simplify any
/// `x0` source path before materializing a `PreparedInput`.
struct PreparedInput<'a> {
    src: ValueLoc<'a>,
}

impl<'a> PreparedInput<'a> {
    /// Returns the x86 GPR to use as the emitted instruction source.
    fn gpr(&self) -> X86Gpr {
        self.src.gpr()
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared input.
    ///
    /// This is the source carrier register code used by instruction encoders.
    /// It is not a RISC-V register index.
    fn id(&self) -> u8 {
        self.gpr().id()
    }
}

/// Prepared architectural destination bound to a computed source value.
///
/// A prepared output must be explicitly written back; dropping one without
/// calling `write_back` is considered a programmer error and panics.
struct PreparedOutput<'a> {
    src: ValueLoc<'a>,
    dest: MapTarget,
    written_back: bool,
}

impl<'a> PreparedOutput<'a> {
    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared output
    /// source carrier.
    ///
    /// This is the source register code used by instruction encoders. It is
    /// not a destination map id and not a RISC-V register index.
    fn id(&self) -> u8 {
        self.src.gpr().id()
    }

    // TODO: allow multiple live PreparedInput/PreparedOutput without re-borrowing Translator;
    // fix unsafe workaround in prepared_output_drop_after_write_back_does_not_panic.

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
    fn write_back(mut self, translator: &mut Translator) {
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

impl Translator {
    /// Creates a new translator from a validated mapping plan.
    ///
    /// # Arguments
    ///
    /// - `emitter`: x86 assembler used for machine code emission
    /// - `plan`: coupled mapping and temp-pool derivation output
    ///
    /// # Panics
    ///
    /// Panics if the derived temporary register set contains duplicates.
    fn new(mut emitter: Assembler, plan: MappingPlan, base_riscv_pc: u64) -> Self {
        let (reg_map, unused_gprs) = plan.into_parts();
        let temp_allocator = TempAllocator::new(unused_gprs);
        let jt_label = emitter.new_dynamic_label();
        Self {
            emitter,
            reg_map,
            temp_allocator,
            cf: ControlFlowState {
                current_riscv_pc: base_riscv_pc,
                base_riscv_pc,
                pc_labels: HashMap::new(),
                jump_table: Vec::new(),
                jt_label,
            },
        }
    }

    /// Prepares a source register operand for emission.
    ///
    /// # Panics
    ///
    /// Panics when called with a source that maps to `ConstZero` (`x0`).
    /// Callers must simplify `x0`-dependent instruction forms before invoking
    /// this path.
    fn prepare_input(&mut self, src: RiscvRegister) -> PreparedInput<'_> {
        match self.reg_map.get(&src) {
            MapTarget::ConstZero => {
                panic!("prepare_input invariant violated: x0/ConstZero must be handled by lowering before prepare_input")
            }
            MapTarget::Gpr(reg) => PreparedInput {
                src: ValueLoc::Mapped(*reg),
            },
            MapTarget::XmmExclusive(reg)
            | MapTarget::XmmShared {
                reg,
                lane: XmmLane::Low,
            } => {
                let temp = self
                    .temp_allocator
                    .allocate()
                    .unwrap_or_else(|_| panic!("prepare_input could not allocate temp GPR"));

                dynasm!(self.emitter; movq Rq(temp.id()), Rx(reg.id()));

                PreparedInput {
                    src: ValueLoc::Temp(temp),
                }
            }
            MapTarget::XmmShared {
                reg,
                lane: XmmLane::High,
            } => {
                let temp = self
                    .temp_allocator
                    .allocate()
                    .unwrap_or_else(|_| panic!("prepare_input could not allocate temp GPR"));

                dynasm!(self.emitter; pextrq Rq(temp.id()), Rx(reg.id()), 1);

                PreparedInput {
                    src: ValueLoc::Temp(temp),
                }
            }
        }
    }

    /// Prepares an architectural destination and source carrier for emission.
    ///
    /// # Panics
    ///
    /// Panics when called with a destination that maps to `ConstZero` (`x0`).
    /// Lowering must handle `rd = x0` paths explicitly and avoid this API.
    fn prepare_output(&mut self, dst: RiscvRegister) -> PreparedOutput<'_> {
        let dest = *self.reg_map.get(&dst);
        let src = match dest {
            MapTarget::ConstZero => {
                panic!("prepare_output invariant violated: x0/ConstZero destination must be handled by lowering before prepare_output")
            }
            MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
            MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                let temp = self
                    .temp_allocator
                    .allocate()
                    .unwrap_or_else(|_| panic!("prepare_output could not allocate temp GPR"));
                ValueLoc::Temp(temp)
            }
        };

        PreparedOutput {
            src,
            dest,
            written_back: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use dynasmrt::x64::Assembler;

    use super::*;

    fn new_translator() -> Translator {
        Translator::new(Assembler::new().unwrap(), RegisterMapping::default_plan(), 0)
    }

    #[test]
    #[should_panic(expected = "prepare_input invariant violated: x0/ConstZero")]
    fn prepare_input_panics_on_x0() {
        let mut translator = new_translator();
        let _ = translator.prepare_input(RiscvRegister::Zero);
    }

    #[test]
    #[should_panic(expected = "prepare_output invariant violated: x0/ConstZero")]
    fn prepare_output_panics_on_x0() {
        let mut translator = new_translator();
        let _ = translator.prepare_output(RiscvRegister::Zero);
    }

    #[test]
    #[should_panic(expected = "PreparedOutput dropped before write_back")]
    fn prepared_output_drop_without_write_back_panics() {
        let mut translator = new_translator();
        let _ = translator.prepare_output(RiscvRegister::A0);
    }

    #[test]
    fn prepare_input_gpr_returns_mapped_id() {
        let mut translator = new_translator();
        let input = translator.prepare_input(RiscvRegister::A0);
        assert_eq!(input.id(), X86Gpr::Rdi.id());
    }

    #[test]
    fn prepare_output_gpr_uses_mapped_source_id() {
        let mut translator = new_translator();
        let mut out = translator.prepare_output(RiscvRegister::A0);
        assert_eq!(out.id(), X86Gpr::Rdi.id());
        out.written_back = true;
    }

    #[test]
    fn prepared_output_drop_after_write_back_does_not_panic() {
        let mut translator = new_translator();
        let translator_ptr: *mut Translator = &mut translator;
        let out = translator.prepare_output(RiscvRegister::A0);
        unsafe { out.write_back(&mut *translator_ptr) };
    }
}
