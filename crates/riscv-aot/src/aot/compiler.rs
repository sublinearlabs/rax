use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use dynasmrt::x64::Assembler;

use crate::aot::register_mapping::RegisterMapping;
use crate::aot::translator::Translator;
use riscv_core::decode::{decode, Instruction};
use riscv_elfgen::elfgen::analyzer::{analyze_elf, AnalyzeElfError};
use riscv_elfgen::elfgen::emitter::EmitElfError;

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
    let bytes = fs::read(input_path.as_ref())?;
    let mut layout = analyze_elf(&bytes)?;
    let insns = decode_insns(&layout.executable_segment().data);

    let assembler = Assembler::new()?;
    let mut translator = Translator::new(
        assembler,
        RegisterMapping::default_plan(),
        layout.source_executable_vaddr,
        layout.executable_segment().vaddr,
    );
    translator.translate_insns(&insns);
    let translated_entry = translator.x86_vaddr_for_riscv_pc(layout.source_entry_vaddr);
    let x86_bytes = translator.finalize();

    layout.replace_executable(x86_bytes, translated_entry);
    let elf_bytes = layout.emit_x86_elf()?;

    fs::write(output_path.as_ref(), elf_bytes)?;
    fs::set_permissions(output_path.as_ref(), fs::Permissions::from_mode(0o755))?;

    Ok(())
}
