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
    /// Virtual address where the translated x86 code segment will be loaded.
    ///
    /// This is distinct from `base_riscv_pc` because `base_riscv_pc` is source
    /// program state, while this is the runtime address used for emitted x86
    /// absolute targets.
    pub(super) base_x86_vaddr: u64,
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
    pub(super) fn new(
        mut emitter: Assembler,
        plan: MappingPlan,
        base_riscv_pc: u64,
        base_x86_vaddr: u64,
    ) -> Self {
        let (reg_map, unused_gprs) = plan.into_parts();
        let jt_label = emitter.new_dynamic_label();
        Self {
            emitter,
            reg_map,
            unused_gprs,
            cf: ControlFlowState {
                current_riscv_pc: base_riscv_pc,
                base_riscv_pc,
                base_x86_vaddr,
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
    pub(super) fn translate_insns(&mut self, insns: &[Instruction]) {
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
            .map(|offset| offset.0 + self.cf.base_x86_vaddr as usize)
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

    /// Returns the translated x86 virtual address for a source RISC-V PC.
    pub(super) fn x86_vaddr_for_riscv_pc(&self, riscv_pc: u64) -> u64 {
        let slot = (riscv_pc - self.cf.base_riscv_pc) / 4;
        self.cf.base_x86_vaddr + self.cf.riscv_pc_to_x86_offset[slot as usize].0 as u64
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
    use std::io::ErrorKind;
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    use dynasmrt::x64::Assembler;

    use crate::decode::{Instruction, I};

    use super::*;

    static AOT_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn maps_non_base_source_pc_to_translated_x86_vaddr() {
        let assembler = Assembler::new().expect("failed to create x86 assembler");
        let mut translator = Translator::new(
            assembler,
            RegisterMapping::default_plan(),
            0x400000,
            0x600000,
        );
        let insns = [
            Instruction::Addi(I {
                rd: 1,
                rs1: 0,
                imm: 1,
            }),
            Instruction::Addi(I {
                rd: 2,
                rs1: 0,
                imm: 2,
            }),
        ];

        translator.translate_insns(&insns);

        assert_eq!(translator.x86_vaddr_for_riscv_pc(0x400000), 0x600000);
        assert!(translator.x86_vaddr_for_riscv_pc(0x400004) > 0x600000);
    }

    /// Generate an equivalent x86 ELF file given a RISC-V ELF path.
    ///
    /// Delegates to the public [`crate::aot::compiler::compile_elf_file`] API.
    fn compile_elf_for_test(path: &str, out_path: &str) {
        crate::aot::compiler::compile_elf_file(path, out_path).expect("AOT compilation failed");
    }

    fn compile_and_run_aot(
        name: &str,
        elf_path: &str,
        stdin_input: Option<&[u8]>,
        expected_stdout: Option<&[u8]>,
    ) {
        let id = AOT_TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let out_dir = std::env::temp_dir().join(format!(
            "riscv_aot_test_{}_{}_{}",
            std::process::id(),
            name,
            id
        ));
        fs::create_dir(&out_dir).expect("failed to create AOT temp directory");
        let out_path = out_dir.join("aot-bin");
        let out_path_str = out_path.to_str().expect("temp path is not valid UTF-8");

        compile_elf_for_test(elf_path, out_path_str);

        let mut command = Command::new(&out_path);
        command.stdin(if stdin_input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut retries_remaining = 50;
        let mut child = loop {
            match command.spawn() {
                Ok(child) => break child,
                Err(err)
                    if err.kind() == ErrorKind::ExecutableFileBusy && retries_remaining > 0 =>
                {
                    retries_remaining -= 1;
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("failed to spawn AOT binary: {err}"),
            }
        };
        if let Some(input) = stdin_input {
            child
                .stdin
                .take()
                .expect("failed to open AOT stdin")
                .write_all(input)
                .expect("failed to write AOT stdin");
        }

        let output = child
            .wait_with_output()
            .expect("failed to wait on AOT binary");
        let _ = fs::remove_file(&out_path);
        let _ = fs::remove_dir(&out_dir);

        let status_code = output.status.code();
        if let Some(code) = status_code {
            if code != 0 {
                println!("failing test {}", code >> 1);
            }
        }

        assert!(
            output.status.success(),
            "AOT binary failed: status={:?}, failing_test={:?}\nstdout:\n{}\nstderr:\n{}",
            status_code,
            status_code.map(|code| code >> 1),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        if let Some(expected) = expected_stdout {
            assert_eq!(output.stdout.as_slice(), expected, "AOT stdout mismatch");
        }
    }

    fn compile_and_run_aot_dir(dir: &str) {
        let mut paths = fs::read_dir(dir)
            .expect("failed to read AOT test directory")
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();

        paths.sort();

        for path in paths {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("AOT test path has invalid file name");

            if name == "rv64ui-p-fence_i" {
                // This test relies on self-modifying code plus FENCE.I. The current
                // AOT pipeline translates code once and has no invalidation/retranslation.
                println!("skipping AOT test: {}", path.display());
                continue;
            }

            if name == "rv64ua-p-lrsc" {
                // LR/SC requires tracking reservation state across instructions.
                // Current AOT only lowers simple AMO RMW operations, so skip until
                // the A extension is handled properly.
                println!("skipping AOT test: {}", path.display());
                continue;
            }

            println!("running AOT test: {}", path.display());
            let path = path.to_str().expect("AOT test path is not valid UTF-8");

            compile_and_run_aot(name, path, None, None);
        }
    }

    #[test]
    fn compile_echo_ima_writes_output_elf() {
        compile_elf_for_test("test-bin/rust-bin/echo/echo-ima", "test-bin/output_echo");

        let out = fs::metadata("test-bin/output_echo").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }

    #[test]
    fn compile_fib_ima_writes_output_elf() {
        compile_elf_for_test("test-bin/rust-bin/fib/fib-ima", "test-bin/output_fib");

        let out = fs::metadata("test-bin/output_fib").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }

    #[test]
    fn compile_exec_block_ima_writes_output_elf() {
        compile_elf_for_test(
            "test-bin/rust-bin/exec-block/exec-block-ima",
            "test-bin/output_exec_block",
        );

        let out = fs::metadata("test-bin/output_exec_block").expect("missing output ELF");
        assert!(out.len() > 0, "output ELF should not be empty");
    }

    #[test]
    fn aot_fib_ima() {
        compile_and_run_aot("fib", "test-bin/rust-bin/fib/fib-ima", None, None);
    }

    #[test]
    fn aot_echo_ima() {
        let input = "Hola Riscv, buenos días".as_bytes();
        compile_and_run_aot(
            "echo",
            "test-bin/rust-bin/echo/echo-ima",
            Some(input),
            Some(input),
        );
    }

    #[test]
    fn aot_exec_block_ima() {
        let input_hex = fs::read_to_string("examples/exec-block.input")
            .expect("failed to read exec-block input");
        let input = hex::decode(input_hex.trim()).expect("failed to decode exec-block input");

        compile_and_run_aot(
            "exec_block",
            "test-bin/rust-bin/exec-block/exec-block-ima",
            Some(&input),
            None,
        );
    }

    #[test]
    fn aot_rv64ui() {
        compile_and_run_aot_dir("test-bin/rv64ui");
    }

    #[cfg(feature = "ext_m")]
    #[test]
    fn aot_rv64um() {
        compile_and_run_aot_dir("test-bin/rv64um");
    }

    #[cfg(feature = "ext_a")]
    #[test]
    fn aot_rv64ua() {
        compile_and_run_aot_dir("test-bin/rv64ua");
    }
}
