use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use dynasmrt::x64::Assembler;
use iced_x86::{Decoder, DecoderOptions};

use crate::aot::register_mapping::RegisterMapping;
use crate::aot::translator::Translator;
use riscv_core::decode::{decode, Instruction};
use riscv_elfgen::elfgen::analyzer::{analyze_elf, AnalyzeElfError};
use riscv_elfgen::elfgen::emitter::EmitElfError;

#[derive(Debug, Clone)]
pub struct AotCompileStats {
    pub riscv_instruction_count: usize,
    pub x86_instruction_count: usize,
    pub x86_code_bytes: usize,
    pub jump_table_bytes: usize,
}

impl AotCompileStats {
    pub fn x86_instructions_per_riscv_instruction(&self) -> f64 {
        if self.riscv_instruction_count == 0 {
            return 0.0;
        }

        self.x86_instruction_count as f64 / self.riscv_instruction_count as f64
    }
}

#[derive(Debug)]
pub enum AotCompileError {
    Io(io::Error),
    AnalyzeElf(AnalyzeElfError),
    EmitElf(EmitElfError),
}

impl std::fmt::Display for AotCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::AnalyzeElf(err) => write!(f, "ELF analysis error: {err:?}"),
            Self::EmitElf(err) => write!(f, "ELF emission error: {err:?}"),
        }
    }
}

impl std::error::Error for AotCompileError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::AnalyzeElf(_) | Self::EmitElf(_) => None,
        }
    }
}

impl From<io::Error> for AotCompileError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<AnalyzeElfError> for AotCompileError {
    fn from(err: AnalyzeElfError) -> Self {
        Self::AnalyzeElf(err)
    }
}

impl From<EmitElfError> for AotCompileError {
    fn from(err: EmitElfError) -> Self {
        Self::EmitElf(err)
    }
}

fn decode_insns(bytes: &[u8]) -> Vec<Instruction> {
    bytes
        .chunks(4)
        .map(|chunk| {
            let mut insn_bytes = [0u8; 4];
            insn_bytes[..chunk.len()].copy_from_slice(chunk);
            decode(u32::from_le_bytes(insn_bytes))
        })
        .collect()
}

fn count_x86_instructions(bytes: &[u8], base_x86_vaddr: u64) -> usize {
    let mut decoder = Decoder::with_ip(64, bytes, base_x86_vaddr, DecoderOptions::NONE);
    let mut count = 0;

    while decoder.can_decode() {
        let _ = decoder.decode();
        count += 1;
    }

    count
}

/// Compiles a RISC-V ELF binary to a native x86-64 ELF binary.
///
/// The input must be a statically-linked RISC-V ELF executable with
/// uncompressed (RV64G) instructions. The output is a standalone x86-64
/// ELF executable that produces the same observable behavior.
///
/// # Errors
///
/// Returns [`AotCompileError`] if the input cannot be read, is not a
/// valid RISC-V ELF, or if the output ELF cannot be written.
pub fn compile_elf_file(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), AotCompileError> {
    compile_elf_file_with_stats(input_path, output_path).map(|_| ())
}

pub fn compile_elf_file_with_stats(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<AotCompileStats, AotCompileError> {
    let bytes = fs::read(input_path.as_ref())?;
    let mut layout = analyze_elf(&bytes)?;
    let insns = decode_insns(&layout.executable_segment().data);
    let base_x86_vaddr = layout.executable_segment().vaddr;

    let assembler = Assembler::new()?;
    let mut translator = Translator::new(
        assembler,
        RegisterMapping::default_plan(),
        layout.source_executable_vaddr,
        base_x86_vaddr,
    );
    translator.translate_insns(&insns);
    let code_end_offset = translator.code_end_offset();
    let translated_entry = translator.x86_vaddr_for_riscv_pc(layout.source_entry_vaddr);
    let x86_bytes = translator.finalize();
    let x86_instruction_count =
        count_x86_instructions(&x86_bytes[..code_end_offset], base_x86_vaddr);
    let stats = AotCompileStats {
        riscv_instruction_count: insns.len(),
        x86_instruction_count,
        x86_code_bytes: code_end_offset,
        jump_table_bytes: x86_bytes.len() - code_end_offset,
    };

    layout.replace_executable(x86_bytes, translated_entry);
    let elf_bytes = layout.emit_x86_elf()?;

    fs::write(output_path.as_ref(), elf_bytes)?;
    fs::set_permissions(output_path.as_ref(), fs::Permissions::from_mode(0o755))?;

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn workspace_path(path: &str) -> PathBuf {
        let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/.."));
        root.canonicalize().unwrap().join(path)
    }

    #[test]
    fn compile_elf_file_with_stats_excludes_jump_table() {
        let input = workspace_path("test-bin/rust-bin/fib/fib-ima");
        let output = std::env::temp_dir().join(format!("riscv-aot-stats-{}", std::process::id()));

        let stats = compile_elf_file_with_stats(&input, &output).expect("AOT compilation failed");
        let _ = fs::remove_file(&output);

        assert!(stats.riscv_instruction_count > 0);
        assert!(stats.x86_instruction_count > 0);
        assert!(stats.x86_instructions_per_riscv_instruction() > 0.0);
        assert_eq!(stats.jump_table_bytes, stats.riscv_instruction_count * 8);
    }
}
