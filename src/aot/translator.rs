use std::{collections::HashMap, rc::Rc};

use dynasmrt::{dynasm, x64::Assembler, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi};

use crate::aot::{
    emission,
    register_mapping::{MapTarget, MappingPlan, RegisterMapping, XmmLane},
    registers::{RiscvRegister, X86Gpr},
    temp_alloc::{AllocatedTemp, TempAllocator},
};
use crate::decode::Instruction;

/// AOT translator state used while lowering RISC-V instructions to x86.
///
/// This type owns the emitter and all translation-local state required to
/// materialize inputs and stage outputs for architectural write-back.
pub(super) struct Translator {
    pub(crate) emitter: Assembler,
    reg_map: RegisterMapping,
    unused_gprs: Vec<X86Gpr>,
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
    /// This is the RISC-V PC for slot 0 in `riscv_pc_to_x86_offset` and in
    /// the emitted runtime jump table.
    base_riscv_pc: u64,
    /// Sparse map of direct control-flow target PCs to x86 labels.
    ///
    /// Only PCs that are targets of direct jumps/branches are present.
    direct_target_labels: HashMap<u64, DynamicLabel>,
    /// Dense mapping from RISC-V PC slot to x86 instruction-start offset.
    ///
    /// Slot index is `(pc - base_riscv_pc) / 4` in the no-compressed model.
    riscv_pc_to_x86_offset: Vec<AssemblyOffset>,
    /// Dynamic label that marks the start of the emitted jump-table data.
    ///
    /// Indirect jump paths use this as the base for indexed table lookups.
    jt_label: DynamicLabel,
}

/// Canonical location for a value currently usable as a GPR source.
///
/// Values may already reside in a mapped x86 register or be materialized into
/// a temporary register managed by `TempAllocator`.
#[derive(Clone)]
enum ValueLoc<'a> {
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
}

/// Prepared input ready for instruction emission.
///
/// Inputs may be materialized from mapped GPRs/XMM lanes or represented as
/// `ConstZero` when the architectural source is `x0`.
#[derive(Clone)]
pub(crate) struct PreparedInput<'a> {
    src: ValueLoc<'a>,
}

impl<'a> PreparedInput<'a> {
    /// Returns whether this prepared input represents architectural zero (`x0`).
    ///
    /// Callers should use this guard before requesting a concrete carrier via
    /// `gpr()`/`id()`.
    pub(crate) fn is_zero(&self) -> bool {
        matches!(self.src, ValueLoc::ConstZero)
    }

    /// Returns the x86 GPR to use as the emitted instruction source.
    ///
    /// # Panics
    ///
    /// Panics when this input is `ConstZero`.
    /// Callers must branch on zero-valued inputs before calling this method.
    fn gpr(&self) -> X86Gpr {
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
    pub(crate) fn id(&self) -> u8 {
        self.gpr().id()
    }
}

/// Prepared architectural destination bound to a computed source value.
///
/// A prepared output must be explicitly written back; dropping one without
/// calling `write_back` is considered a programmer error and panics.
pub(crate) struct PreparedOutput<'a> {
    src: ValueLoc<'a>,
    dest: MapTarget,
    written_back: bool,
}

impl<'a> PreparedOutput<'a> {
    /// Returns whether this prepared output targets architectural zero (`x0`).
    ///
    /// A zero output represents an elided architectural write destination and
    /// does not permit carrier-id lookup or write-back emission.
    pub(crate) fn is_zero(&self) -> bool {
        matches!(self.dest, MapTarget::ConstZero)
    }

    /// Returns the x86-64 GPR encoding id (`0..=15`) of this prepared output
    /// source carrier.
    ///
    /// This is the source register code used by instruction encoders. It is
    /// not a destination map id and not a RISC-V register index.
    pub(crate) fn id(&self) -> u8 {
        if self.is_zero() {
            panic!("PreparedOutput::id called on zero/elided output");
        }
        self.src.gpr().id()
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
    pub(crate) fn write_back(mut self, translator: &mut Translator) {
        if self.is_zero() {
            panic!("PreparedOutput::write_back called on zero/elided output");
        }
        let src = self.id();
        match self.dest {
            MapTarget::ConstZero => {
                unreachable!("zero/elided output is rejected before write-back dispatch")
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

    /// Commits an unchanged prepared output without emitting write-back code.
    ///
    /// # Contract
    ///
    /// Callers must guarantee that the architectural destination already
    /// contains the correct result value for this instruction.
    ///
    /// This marks write-back as completed and emits no instructions.
    pub(crate) fn commit_unchanged(mut self) {
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
        let jt_label = emitter.new_dynamic_label();
        Self {
            emitter,
            reg_map,
            unused_gprs,
            cf: ControlFlowState {
                current_riscv_pc: base_riscv_pc,
                base_riscv_pc,
                direct_target_labels: HashMap::new(),
                riscv_pc_to_x86_offset: Vec::new(),
                jt_label,
            },
        }
    }

    /// Builds a temp-register allocator from this translator's temp GPR list.
    ///
    /// Use one allocator while lowering an instruction and pass it to helper
    /// emission functions. Temps are released automatically when their
    /// `AllocatedTemp` values are dropped.
    ///
    /// # Panics
    ///
    /// Panics if the temp register list contains duplicates.
    fn temp_allocator(&self) -> TempAllocator {
        TempAllocator::new(self.unused_gprs.clone())
    }

    /// Converts decoded RISC-V instructions to x86 and finalizes control-flow metadata.
    ///
    /// This v1 path assumes a non-compressed input stream, so the RISC-V PC
    /// advances by 4 bytes per instruction.
    fn translate_insns(&mut self, insns: &[Instruction]) {
        // Record instruction start offsets by PC slot, then translate.
        for insn in insns {
            self.cf.riscv_pc_to_x86_offset.push(self.emitter.offset());
            self.translate_insn(insn);
            self.cf.current_riscv_pc = self.cf.current_riscv_pc.wrapping_add(4);
        }

        // Resolve dynamic labels after instruction offsets are known.
        for (pc, label) in &self.cf.direct_target_labels {
            let riscv_pc_slot = (pc - self.cf.base_riscv_pc) / 4;
            self.emitter
                .labels_mut()
                .define_dynamic(
                    *label,
                    self.cf.riscv_pc_to_x86_offset[riscv_pc_slot as usize],
                )
                .expect("failed to define dynamic label");
        }

        // Build absolute jump targets from `riscv_pc_to_x86_offset` and emit
        // them as the runtime jump table at the end of the generated code.
        // For now, this assumes one contiguous code segment based at
        // `base_riscv_pc`.
        let jump_table_abs_addrs = self
            .cf
            .riscv_pc_to_x86_offset
            .iter()
            .map(|offset| offset.0 + self.cf.base_riscv_pc as usize)
            .collect::<Vec<_>>();

        dynasm!(self.emitter ; =>self.cf.jt_label);
        for target_pc in jump_table_abs_addrs {
            dynasm!(self.emitter; .i64 target_pc as i64);
        }
    }

    fn translate_insn(&mut self, insn: &Instruction) {
        let temps = self.temp_allocator();
        emission::emit_instruction(self, &temps, insn);
    }

    /// Consumes the translator and returns the emitted machine code bytes.
    pub(crate) fn finalize(self) -> Vec<u8> {
        let buf = self.emitter.finalize().unwrap();
        buf.to_vec()
    }

    /// Prepares a fixed set of architectural inputs for one instruction.
    ///
    /// This helper performs operand-level simplification and materialization:
    /// - `x0` inputs are represented as `ConstZero` and emit no materialization
    ///   instructions.
    /// - Duplicate architectural inputs reuse the first prepared value.
    /// - Inputs mapped to XMM locations are materialized once into a temp GPR
    ///   and shared across duplicates through `Rc` ownership.
    ///
    /// # Panics
    ///
    /// Panics when a required temp GPR cannot be allocated.
    pub(crate) fn prepare_inputs<'a, const N: usize>(
        &mut self,
        inputs: [RiscvRegister; N],
        temp_allocator: &'a TempAllocator,
    ) -> [PreparedInput<'a>; N] {
        let mut prepared_inputs: Vec<PreparedInput<'a>> = Vec::with_capacity(N);
        let mut seen = Vec::with_capacity(N);

        for src in inputs {
            // here we should check if we have seen this input before
            let has_seen = seen.iter().position(|reg| *reg == src);

            // push every value into the seen vector
            seen.push(src);

            if let Some(index) = has_seen {
                prepared_inputs.push(prepared_inputs[index].clone());
                continue;
            }

            if src.is_zero() {
                // we avoid emissions for the zero register
                prepared_inputs.push(PreparedInput {
                    src: ValueLoc::ConstZero,
                });
                continue;
            }

            match self.reg_map.get(&src) {
                MapTarget::ConstZero => {
                    unreachable!("we handled the zero case above")
                }
                MapTarget::Gpr(reg) => prepared_inputs.push(PreparedInput {
                    src: ValueLoc::Mapped(*reg),
                }),
                MapTarget::XmmExclusive(reg)
                | MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::Low,
                } => {
                    let temp = temp_allocator
                        .allocate()
                        .unwrap_or_else(|_| panic!("prepare_input could not allocate temp GPR"));

                    dynasm!(self.emitter ; movq Rq(temp.id()), Rx(reg.id()));

                    prepared_inputs.push(PreparedInput {
                        src: ValueLoc::Temp(Rc::new(temp)),
                    });
                }
                MapTarget::XmmShared {
                    reg,
                    lane: XmmLane::High,
                } => {
                    let temp = temp_allocator
                        .allocate()
                        .unwrap_or_else(|_| panic!("prepare_input could not allocate temp GPR"));
                    dynasm!(self.emitter ; pextrq Rq(temp.id()), Rx(reg.id()), 1);
                    prepared_inputs.push(PreparedInput {
                        src: ValueLoc::Temp(Rc::new(temp)),
                    })
                }
            }
        }

        prepared_inputs.try_into().unwrap_or_else(|_| {
            unreachable!("we push one prepared_input for each input, hence should be the same size")
        })
    }

    /// Prepares an architectural destination and source carrier for emission.
    ///
    /// For `x0` destinations this returns a zero/elided output, which must not
    /// be used as a real write-back carrier.
    pub(crate) fn prepare_output<'a>(
        &mut self,
        dst: RiscvRegister,
        temp_allocator: &'a TempAllocator,
    ) -> PreparedOutput<'a> {
        let dest = *self.reg_map.get(&dst);
        let src = match dest {
            MapTarget::ConstZero => ValueLoc::ConstZero,
            MapTarget::Gpr(gpr) => ValueLoc::Mapped(gpr),
            MapTarget::XmmShared { .. } | MapTarget::XmmExclusive(..) => {
                let temp = temp_allocator
                    .allocate()
                    .unwrap_or_else(|_| panic!("prepare_output could not allocate temp GPR"));
                ValueLoc::Temp(Rc::new(temp))
            }
        };

        PreparedOutput {
            src,
            dest,
            written_back: matches!(dest, MapTarget::ConstZero),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use dynasmrt::x64::Assembler;

    use crate::aot::registers::X86Xmm;
    use crate::elf::parse_elf;
    use crate::elf_gen::generate_elf;
    use crate::elf_gen::x86_elf::X86Elf;

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

        let plan = builder
            .build()
            .expect("builder should produce valid mapping");
        Translator::new(Assembler::new().unwrap(), plan, 0)
    }

    #[test]
    fn prepare_output_x0_returns_zero_output() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let out = translator.prepare_output(RiscvRegister::Zero, &temps);
        assert!(out.is_zero());
    }

    #[test]
    #[should_panic(expected = "PreparedOutput::id called on zero/elided output")]
    fn prepare_output_id_panics_on_zero_output() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let out = translator.prepare_output(RiscvRegister::Zero, &temps);
        let _ = out.id();
    }

    #[test]
    #[should_panic(expected = "PreparedOutput::write_back called on zero/elided output")]
    fn prepare_output_write_back_panics_on_zero_output() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let out = translator.prepare_output(RiscvRegister::Zero, &temps);
        out.write_back(&mut translator);
    }

    #[test]
    fn prepare_output_zero_drop_does_not_panic() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let _ = translator.prepare_output(RiscvRegister::Zero, &temps);
    }

    #[test]
    #[should_panic(expected = "PreparedOutput dropped before write_back")]
    fn prepared_output_drop_without_write_back_panics() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let _ = translator.prepare_output(RiscvRegister::A0, &temps);
    }

    #[test]
    fn prepare_output_gpr_uses_mapped_source_id() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let out = translator.prepare_output(RiscvRegister::A0, &temps);
        assert_eq!(out.id(), X86Gpr::Rdi.id());
        out.write_back(&mut translator);
    }

    #[test]
    fn prepared_output_drop_after_write_back_does_not_panic() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let out = translator.prepare_output(RiscvRegister::A0, &temps);
        out.write_back(&mut translator);
    }

    #[test]
    fn prepare_inputs_zero_returns_constzero() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::Zero], &temps);
        assert!(matches!(inputs[0].src, ValueLoc::ConstZero));
    }

    #[test]
    fn prepare_inputs_gpr_maps_to_expected_id() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::A0], &temps);
        assert_eq!(inputs[0].id(), X86Gpr::Rdi.id());
    }

    #[test]
    fn prepare_inputs_duplicate_gpr_reuses_same_carrier() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::A0, RiscvRegister::A0], &temps);
        assert_eq!(inputs[0].id(), inputs[1].id());
    }

    #[test]
    fn prepare_inputs_duplicate_zero_reuses_constzero() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::Zero, RiscvRegister::Zero], &temps);
        assert!(matches!(inputs[0].src, ValueLoc::ConstZero));
        assert!(matches!(inputs[1].src, ValueLoc::ConstZero));
    }

    #[test]
    fn prepare_inputs_duplicate_xmm_shares_temp_owner() {
        let mut translator = new_translator_all_xmm_shared();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::A0, RiscvRegister::A0], &temps);

        let (rc0, rc1) = match (&inputs[0].src, &inputs[1].src) {
            (ValueLoc::Temp(rc0), ValueLoc::Temp(rc1)) => (rc0, rc1),
            _ => panic!("expected both prepared inputs to be temp-backed"),
        };

        assert!(Rc::ptr_eq(rc0, rc1));
        assert_eq!(Rc::strong_count(rc0), 2);
    }

    #[test]
    fn prepare_inputs_drop_one_duplicate_keeps_other_valid() {
        let mut translator = new_translator_all_xmm_shared();
        let temps = translator.temp_allocator();
        let [first, second] =
            translator.prepare_inputs([RiscvRegister::A0, RiscvRegister::A0], &temps);

        let second_id = second.id();
        drop(first);
        assert_eq!(second.id(), second_id);
    }

    #[test]
    #[should_panic(expected = "should not materialize const a zero input")]
    fn prepared_input_id_panics_on_constzero() {
        let mut translator = new_translator();
        let temps = translator.temp_allocator();
        let inputs = translator.prepare_inputs([RiscvRegister::Zero], &temps);
        let _ = inputs[0].id();
    }

    /// Generate an equivalent x86 ELF file given a RISC-V ELF path.
    ///
    /// RISC-V ELF constraints:
    /// - uncompressed format (without the C extension)
    /// - single executable code segment
    fn compile_elf_for_test(path: &str, out_path: &str) {
        let bytes = fs::read(path).expect("failed to read input ELF");
        let mut elf = parse_elf(&bytes);

        let mut executable_count = 0u32;
        for segment in &mut elf.segments {
            if !segment.is_executable() {
                continue;
            }

            executable_count = executable_count.wrapping_add(1);
            if executable_count > 1 {
                panic!("translator only supports ELFs with a single executable segment");
            }

            segment.decode();

            let assembler = Assembler::new().expect("failed to create x86 assembler");
            let mut translator =
                Translator::new(assembler, RegisterMapping::default_plan(), elf.global_entry);
            translator.translate_insns(segment.insns());
            let x86_bytes = translator.finalize();

            let mut x86_elf = X86Elf::new(elf.global_entry);
            assert_eq!(segment.entry(), elf.global_entry);
            x86_elf.add_text(x86_bytes, segment.entry(), segment.offset());

            let elf_bytes = generate_elf(&x86_elf).expect("failed to generate x86 ELF");
            fs::write(out_path, elf_bytes).expect("failed to write x86 ELF output");
            fs::set_permissions(out_path, fs::Permissions::from_mode(0o755))
                .expect("failed to set executable permissions");
        }

        assert_eq!(
            executable_count, 1,
            "expected exactly one executable segment"
        );
    }

    #[test]
    #[ignore = "translate_insn is not implemented yet"]
    fn compile_echo_ima_writes_output_elf() {
        compile_elf_for_test("test-bin/rust-bin/echo/echo-ima", "test-bin/output_echo");

        let out = fs::metadata("test-bin/output_echo").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }

    #[test]
    #[ignore = "translate_insn is not implemented yet"]
    fn compile_fib_ima_writes_output_elf() {
        compile_elf_for_test("test-bin/rust-bin/fib/fib-ima", "test-bin/output_fib");

        let out = fs::metadata("test-bin/output_fib").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }

    #[test]
    #[ignore = "translate_insn is not implemented yet"]
    fn compile_exec_block_ima_writes_output_elf() {
        compile_elf_for_test(
            "test-bin/rust-bin/exec-block/exec-block-ima",
            "test-bin/output_exec_block",
        );

        let out = fs::metadata("test-bin/output_exec_block").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }
}
