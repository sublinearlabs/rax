// TODO add documentation
struct R {
    rd: u8,
    rs1: u8,
    rs2: u8,
}

// TODO add documentation
struct I {
    rd: u8,
    rs1: u8,
    imm: i32,
}

// TODO add documentation
struct Sh {
    rd: u8,
    rs1: u8,
    shamt: u8,
}

// TODO add documentation
struct S {
    rs1: u8,
    rs2: u8,
    imm: i32,
}

// TODO add documentation
struct B {
    rs1: u8,
    rs2: u8,
    imm: i32,
}

// TODO add documentation
struct J {
    rd: u8,
    imm: i32,
}

// TODO add documentation
struct U {
    rd: u8,
    imm: i32,
}

enum Instruction {
    // Base Instruction (I)
    // Integer Register Register
    Add(R),
    Sub(R),
    Sll(R),
    Slt(R),
    Sltu(R),
    Xor(R),
    Srl(R),
    Sra(R),
    Or(R),
    And(R),
    // Integer Register Immediate
    Addi(I),
    Slti(I),
    Sltiu(I),
    Xori(I),
    Ori(I),
    Andi(I),
    Slli(Sh),
    Srli(Sh),
    Srai(Sh),
    // Loads
    Lb(I),
    Lh(I),
    Lw(I),
    Lbu(I),
    Lhu(I),
    // Stores
    Sb(S),
    Sh(S),
    Sw(S),
    // Branches
    Beq(B),
    Bne(B),
    Blt(B),
    Bge(B),
    Bltu(B),
    Bgeu(B),
    // Jumps
    Jal(J),
    Jalr(I),
    // Upper Immediates
    Lui(U),
    Auipc(U),
    // System
    Ecall,
    Ebreak,
    // Fence
    Fence,
}
