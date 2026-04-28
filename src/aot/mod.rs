use cranelift_codegen::gimli::Register;
use dynasmrt::x64::{Assembler, Rq};
use iced_x86::{Decoder, DecoderOptions, Formatter, Instruction, NasmFormatter};
use std::fs;

use crate::{
    aot::{
        compiler::Compiler,
        register_mapping::{RegisterLocation, RegisterMapping, XmmLane},
    },
    elf::parse_elf,
    elf_gen::{generate_elf, X86Elf},
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

        // x0(zero)
        RegisterLocation::Zero,
        // x1(ra)
        RegisterLocation::Gpr(Rq::RBX as u8),
        // x2(sp)
        RegisterLocation::Gpr(Rq::RSP as u8),
        // x3(gp)
        RegisterLocation::XmmShared(12, XmmLane::LOWER),
        // x4(tp)
        RegisterLocation::XmmShared(12, XmmLane::UPPER),
        // x5(t0)
        RegisterLocation::Gpr(Rq::R14 as u8),
        // x6(t1)
        RegisterLocation::Gpr(Rq::R15 as u8),
        // x7(t2)
        RegisterLocation::Gpr(Rq::RBP as u8),
        // x8(s0/fp)
        RegisterLocation::Xmm(1),
        // x9(s1)
        RegisterLocation::Xmm(2),
        // x10(a0)
        RegisterLocation::Gpr(Rq::RDI as u8),
        // x11(a1)
        RegisterLocation::Gpr(Rq::RSI as u8),
        // x12(a2)
        RegisterLocation::Gpr(Rq::RDX as u8),
        // x13(a3)
        RegisterLocation::Gpr(Rq::R10 as u8),
        // x14(a4)
        RegisterLocation::Gpr(Rq::R8 as u8),
        // x15(a5)
        RegisterLocation::Gpr(Rq::R9 as u8),
        // x16(a6)
        RegisterLocation::Xmm(3),
        // x17(a7)
        RegisterLocation::Gpr(Rq::RAX as u8),
        // x18(s2)
        RegisterLocation::Xmm(4),
        // x19(s3)
        RegisterLocation::Xmm(5),
        // x20(s4)
        RegisterLocation::Xmm(6),
        // x21(s5)
        RegisterLocation::Xmm(7),
        // x22(s6)
        RegisterLocation::Xmm(8),
        // x23(s7)
        RegisterLocation::Xmm(9),
        // x24(s8)
        RegisterLocation::Xmm(10),
        // x25(s9)
        RegisterLocation::Xmm(11),
        // x26(s10)
        RegisterLocation::XmmShared(13, XmmLane::LOWER),
        // x27(s11)
        RegisterLocation::XmmShared(13, XmmLane::UPPER),
        // x28(t3)
        RegisterLocation::XmmShared(14, XmmLane::LOWER),
        // x29(t4)
        RegisterLocation::XmmShared(14, XmmLane::UPPER),
        // x30(t5)
        RegisterLocation::XmmShared(15, XmmLane::LOWER),
        // x31(t6)
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
            disassemble_x64(&bytes, elf.global_entry);

            let mut x86_elf = X86Elf::new(elf.global_entry);
            assert!(segment.entry == elf.global_entry);
            x86_elf.add_text(bytes, segment.entry, segment.offset);
            let elf_bytes = generate_elf(&x86_elf).unwrap();

            fs::write("./test-bin/output.elf", elf_bytes).unwrap();
        }
    }
}

fn disassemble_x64(code: &[u8], base_ip: u64) {
    let mut decoder = Decoder::with_ip(64, code, base_ip, DecoderOptions::NONE);
    let mut formatter = NasmFormatter::new();
    let mut instr = Instruction::default();
    let mut out = String::new();

    let mut insn_count = 0;

    while decoder.can_decode() {
        decoder.decode_out(&mut instr);
        out.clear();
        formatter.format(&instr, &mut out);
        println!("{:016X} {}", instr.ip(), out);
        insn_count += 1;
    }

    println!("x86 insn_count: {}", insn_count);
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
