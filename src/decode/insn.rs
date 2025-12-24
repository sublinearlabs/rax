use crate::decode::insn_formats::{B, I, J, R, R4, RF, S, Sh, U};

pub(crate) enum Instruction {
    // RV32I
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

    // RV64I
    // Register-Register
    Addw(R),
    Subw(R),
    Sllw(R),
    Srlw(R),
    Sraw(R),
    // Register-Immediate
    Addiw(I),
    Slliw(Sh),
    Srliw(Sh),
    Sraiw(Sh),
    // Loads
    Ld(I),
    Lwu(I),
    // Stores
    Sd(S),

    // M extension
    // RV32M
    Mul(R),
    Mulh(R),
    Mulhsu(R),
    Mulhu(R),
    Div(R),
    Divu(R),
    Rem(R),
    Remu(R),

    // RV64M
    Mulw(R),
    Divw(R),
    Divuw(R),
    Remw(R),
    Remuw(R),

    // A extension
    // RV32A
    LrW(R),
    ScW(R),
    AmoSwapW(R),
    AmoAddW(R),
    AmoXorW(R),
    AmoAndW(R),
    AmoOrW(R),
    AmoMinW(R),
    AmoMaxW(R),
    AmoMinuW(R),
    AmoMaxuW(R),

    // RV64A
    LrD(R),
    ScD(R),
    AmoSwapD(R),
    AmoAddD(R),
    AmoXorD(R),
    AmoAndD(R),
    AmoOrD(R),
    AmoMinD(R),
    AmoMaxD(R),
    AmoMinuD(R),
    AmoMaxuD(R),

    // F extension
    // RV32F
    Flw(I),
    Fsw(S),
    FmaddS(R4),
    FmsubS(R4),
    FnmsubS(R4),
    FnmaddS(R4),
    FaddS(RF),
    FsubS(RF),
    FmulS(RF),
    FdivS(RF),
    FsqrtS(RF),
    FsgnjS(RF),
    FsgnjnS(RF),
    FsgnjxS(RF),
    FminS(RF),
    FmaxS(RF),
    FeqS(RF),
    FltS(RF),
    FleS(RF),
    FcvtWS(RF),
    FcvtWuS(RF),
    FcvtSW(RF),
    FcvtSWu(RF),
    FmvXW(RF),
    FmvWX(RF),
    FclassS(RF),

    // RV64F
    FcvtLS(RF),
    FcvtLuS(RF),
    FcvtSL(RF),
    FcvtSLu(RF),

    // D extension
    // RV32D
    Fld(I),
    Fsd(S),
    FmaddD(R4),
    FmsubD(R4),
    FnmsubD(R4),
    FnmaddD(R4),
    FaddD(RF),
    FsubD(RF),
    FmulD(RF),
    FdivD(RF),
    FsqrtD(RF),
    FsgnjD(RF),
    FsgnjnD(RF),
    FsgnjxD(RF),
    FminD(RF),
    FmaxD(RF),
    FeqD(RF),
    FltD(RF),
    FleD(RF),
    FcvtWD(RF),
    FcvtWuD(RF),
    FcvtDW(RF),
    FcvtDWu(RF),
    FcvtSD(RF),
    FcvtDS(RF),
    FclassD(RF),

    // RV64D
    FcvtLD(RF),
    FcvtLuD(RF),
    FcvtDL(RF),
    FcvtDLu(RF),
    FmvXD(RF),
    FmvDX(RF),

    // Zicsr
    // CSR Register
    Csrrw(I),
    Csrrs(I),
    Csrrc(I),

    // CSR Register Immediate
    Csrrwi(I),
    Csrrsi(I),
    Csrrci(I),

    // Illegal Instruction
    Illegal(u32),
}
