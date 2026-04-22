use dynasmrt::x64::{Assembler, Rq};
use std::fs;

use crate::{
    aot::{
        compiler::Compiler,
        register_mapping::{RegisterLocation, RegisterMapping, XmmLane},
    },
    elf::parse_elf,
};

pub(crate) mod compiler;
pub(crate) mod register_mapping;

/// Define register mapping
const REGISTER_MAPPING: RegisterMapping = RegisterMapping {
    map: [
        // TODO: this should be RegisterLocation::ConstZero
        // but the current architecture doesn't support this
        // I need to refactor the approach based on the new
        // things learnt.
        RegisterLocation::XmmShared(11, XmmLane::UPPER),
        RegisterLocation::Gpr(Rq::RBX as u8),
        RegisterLocation::Gpr(Rq::RSP as u8),
        RegisterLocation::XmmShared(12, XmmLane::LOWER),
        RegisterLocation::XmmShared(12, XmmLane::UPPER),
        RegisterLocation::Gpr(Rq::R14 as u8),
        RegisterLocation::Gpr(Rq::R15 as u8),
        RegisterLocation::Gpr(Rq::RBP as u8),
        RegisterLocation::Xmm(1),
        RegisterLocation::Xmm(2),
        RegisterLocation::Gpr(Rq::RDI as u8),
        RegisterLocation::Gpr(Rq::RSI as u8),
        RegisterLocation::Gpr(Rq::RDX as u8),
        RegisterLocation::Gpr(Rq::R10 as u8),
        RegisterLocation::Gpr(Rq::R8 as u8),
        RegisterLocation::Gpr(Rq::R9 as u8),
        RegisterLocation::Xmm(3),
        RegisterLocation::Gpr(Rq::RAX as u8),
        RegisterLocation::Xmm(4),
        RegisterLocation::Xmm(5),
        RegisterLocation::Xmm(6),
        RegisterLocation::Xmm(7),
        RegisterLocation::Xmm(8),
        RegisterLocation::Xmm(9),
        RegisterLocation::Xmm(10),
        // TODO: convert this back to non-shared when you fix the zero register
        RegisterLocation::XmmShared(11, XmmLane::LOWER),
        RegisterLocation::XmmShared(13, XmmLane::LOWER),
        RegisterLocation::XmmShared(13, XmmLane::UPPER),
        RegisterLocation::XmmShared(14, XmmLane::LOWER),
        RegisterLocation::XmmShared(14, XmmLane::UPPER),
        RegisterLocation::XmmShared(15, XmmLane::LOWER),
        RegisterLocation::XmmShared(15, XmmLane::UPPER),
    ],
    temps: [Rq::R12 as u8, Rq::RCX as u8, Rq::R11 as u8],
};

/// Generate an equivalent x86 ELF file given the path to a riscv elf file
///
/// RISCV ELF Constraints
/// - uncompressed format (without the c extension)
/// - single code segment
fn compile_elf(path: &'static str) {
    let bytes = fs::read(path).unwrap();
    let mut elf = parse_elf(&bytes);

    let mut no_executable = 0;

    // find and decode code segments
    for segment in &mut elf.segments {
        if segment.is_executable {
            // ensure that only a single code segment exists in the binary
            // TODO: figure out how to get rid of this restriction
            no_executable += 1;
            if no_executable > 1 {
                panic!("compiler only works with elfs that have a single code segment");
            }

            segment.decode();

            let assembler = Assembler::new().unwrap();
            let mut compiler = Compiler::init(assembler, REGISTER_MAPPING, elf.global_entry);

            compiler.translate_insns(&segment.insns);

            let bytes = compiler.finalize();
            dbg!(bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::aot::compile_elf;

    #[test]
    fn compile_echo_ima() {
        const ECHO_BINARY: &str = "test-bin/rust-bin/echo/echo-ima";
        compile_elf(ECHO_BINARY);
    }
}
