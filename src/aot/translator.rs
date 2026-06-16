use std::collections::HashMap;

use dynasmrt::{dynasm, x64::Assembler, AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi};

use crate::aot::{
    emission,
    register_mapping::{MappingPlan, RegisterMapping},
    registers::X86Gpr,
    temp_alloc::TempAllocator,
};
use crate::decode::Instruction;

/// AOT translator state used while lowering RISC-V instructions to x86.
///
/// This type owns the emitter and all translation-local state required to
/// materialize inputs and stage outputs for architectural write-back.
pub(super) struct Translator {
    pub(super) emitter: Assembler,
    pub(super) reg_map: RegisterMapping,
    unused_gprs: Vec<X86Gpr>,
    pub(super) cf: ControlFlowState,
}

/// Control-flow translation state scoped to a translator instance.
pub(super) struct ControlFlowState {
    /// RISC-V PC of the instruction currently being translated.
    ///
    /// This advances in translation order and is used for instruction-local
    /// control-flow semantics such as computing return PCs for jumps.
    current_riscv_pc: u64,
    /// Anchor PC for the translated RISC-V region.
    ///
    /// This is the RISC-V PC for slot 0 in `riscv_pc_to_x86_offset` and in
    /// the emitted runtime jump table.
    pub(super) base_riscv_pc: u64,
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
    pub(super) jt_label: DynamicLabel,
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
    pub(super) fn new(mut emitter: Assembler, plan: MappingPlan, base_riscv_pc: u64) -> Self {
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

    /// Returns the RISC-V PC of the instruction currently being translated.
    ///
    /// The value is advanced only after instruction emission, so PC-relative
    /// lowerings such as `auipc` observe the source instruction's PC.
    pub(super) fn current_pc(&self) -> u64 {
        self.cf.current_riscv_pc
    }

    /// Returns or Creates a new dynamic label for a riscv pc
    pub(super) fn target_label(&mut self, branch_target: u64) -> DynamicLabel {
        *self
            .cf
            .direct_target_labels
            .entry(branch_target)
            .or_insert_with(|| self.emitter.new_dynamic_label())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use dynasmrt::x64::Assembler;

    use crate::elf::parse_elf;
    use crate::elf_gen::generate_elf;
    use crate::elf_gen::x86_elf::X86Elf;

    use super::*;

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
