// RISCV Opcodes
pub(crate) enum Opcode {
    Add,
    Sub,
    Xor,
    Or,
    And,
    Sll,
    Srl,
    Sra,
    Slt,
    Sltu,

    Addi,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Slti,
    Sltiu,

    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,

    Sb,
    Sh,
    Sw,

    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,

    Jal,
    Jalr,

    Lui,
    Auipc,

    Ecall,
    Ebreak,
    Eother,

    Fence,
}

enum InstructionType {
    R,
    I,
    S,
    B,
    U,
    J,
    FENCE,
}

// RISCV insturction
pub(crate) struct Instruction {
    pub(crate) opcode: Opcode,
    pub(crate) rd: usize,
    pub(crate) rs1: usize,
    pub(crate) rs2: usize,
    pub(crate) imm: u64,
}

impl Instruction {
    pub(crate) fn new(opcode: Opcode) -> Self {
        Self {
            opcode,
            rd: 0,
            rs1: 0,
            rs2: 0,
            imm: 0,
        }
    }

    pub(crate) fn rd(self, val: usize) -> Self {
        Self { rd: val, ..self }
    }

    pub(crate) fn rs1(self, val: usize) -> Self {
        Self { rs1: val, ..self }
    }

    pub(crate) fn rs2(self, val: usize) -> Self {
        Self { rs2: val, ..self }
    }

    pub(crate) fn imm(self, val: u64) -> Self {
        Self { imm: val, ..self }
    }
}

pub(crate) fn decode_insn(insn: u32) -> Instruction {
    todo!()
}
