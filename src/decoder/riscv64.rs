// Copyright (c) <OWNER>

// L2 Decoder: RV64I decode logic. Table-driven; data tables separated from
// logic (see tables.rs). Every instruction keeps its source address via
// `Decoded`. Returns `Error::Decode` for unknown opcodes; never panics.

use crate::bytes::Reader;
use crate::error::{Error, Result};
use crate::decoder::tables::sext;

/// A decoded RV64I instruction. All fields are public for inspection by the
// IR lifter and humanizer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    // RV64I integer register-immediate instructions
    Lb { rd: u8, rs1: u8, imm: i64 },
    Lh { rd: u8, rs1: u8, imm: i64 },
    Lw { rd: u8, rs1: u8, imm: i64 },
    Ld { rd: u8, rs1: u8, imm: i64 },
    Lbu { rd: u8, rs1: u8, imm: i64 },
    Lhu { rd: u8, rs1: u8, imm: i64 },
    Lwu { rd: u8, rs1: u8, imm: i64 },
    Addi { rd: u8, rs1: u8, imm: i64 },
    Slti { rd: u8, rs1: u8, imm: i64 },
    Sltiu { rd: u8, rs1: u8, imm: i64 },
    Xori { rd: u8, rs1: u8, imm: i64 },
    Ori { rd: u8, rs1: u8, imm: i64 },
    Andi { rd: u8, rs1: u8, imm: i64 },
    Slli { rd: u8, rs1: u8, shamt: u64 },
    Srli { rd: u8, rs1: u8, shamt: u64 },
    Srai { rd: u8, rs1: u8, shamt: u64 },

    // RV64I integer register-register instructions
    Add { rd: u8, rs1: u8, rs2: u8 },
    Sub { rd: u8, rs1: u8, rs2: u8 },
    Sll { rd: u8, rs1: u8, rs2: u8 },
    Slt { rd: u8, rs1: u8, rs2: u8 },
    Sltu { rd: u8, rs1: u8, rs2: u8 },
    Xor { rd: u8, rs1: u8, rs2: u8 },
    Srl { rd: u8, rs1: u8, rs2: u8 },
    Sra { rd: u8, rs1: u8, rs2: u8 },
    Or { rd: u8, rs1: u8, rs2: u8 },
    And { rd: u8, rs1: u8, rs2: u8 },

    // RV64I loads / stores
    Sb { rs1: u8, rs2: u8, imm: i64 },
    Sh { rs1: u8, rs2: u8, imm: i64 },
    Sw { rs1: u8, rs2: u8, imm: i64 },
    Sd { rs1: u8, rs2: u8, imm: i64 },

    // RV64I branches
    Beq { rs1: u8, rs2: u8, imm: i64 },
    Bne { rs1: u8, rs2: u8, imm: i64 },
    Blt { rs1: u8, rs2: u8, imm: i64 },
    Bge { rs1: u8, rs2: u8, imm: i64 },
    Bltu { rs1: u8, rs2: u8, imm: i64 },
    Bgeu { rs1: u8, rs2: u8, imm: i64 },

    // RV64I jumps / control
    Jal { rd: u8, imm: i64 },
    Jalr { rd: u8, rs1: u8, imm: i64 },
    Auipc { rd: u8, imm: i64 },
    Lui { rd: u8, imm: i64 },

    // RV64I system / environment
    Ecall,
    Ebreak,
    Csrrw { rd: u8, rs1: u8, csr: u16 },
    Csrrs { rd: u8, rs1: u8, csr: u16 },
    Csrrc { rd: u8, rs1: u8, csr: u16 },
    Csrrwi { rd: u8, zimm: u16, csr: u16 },
    Csrrsi { rd: u8, zimm: u16, csr: u16 },
    Csrrci { rd: u8, zimm: u16, csr: u16 },

    /// An instruction the decoder does not recognize (opcode / funct fields).
    Unknown { opcode: u32 },
}

/// The decoded result, carrying the source address and instruction size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    pub address: u64,
    pub size: usize,
    pub instruction: Instruction,
}

/// Stateless RV64I decoder.
pub struct Decoder {}

impl Decoder {
    pub fn new() -> Self {
        Self {}
    }

    /// Decode one 32-bit RV64I instruction from `bytes` at `address`.
    /// `bytes` must be at least 4 bytes long starting at offset 0.
    pub fn decode(&self, bytes: &[u8], address: u64) -> Result<Decoded> {
        if bytes.len() < 4 {
            return Err(Error::Truncated {
                wanted: 4,
                have: bytes.len(),
                offset: 0,
            });
        }
        let inst = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        decode_instruction(inst, address)
    }
}

impl Default for Decoder {
    fn default() -> Self {
        Self::new()
    }
}

/// Decode a single 32-bit instruction word.
fn decode_instruction(inst: u32, address: u64) -> Result<Decoded> {
    let opcode = inst & 0x7f;
    let rd = ((inst >> 7) & 0x1f) as u8;
    let funct3 = ((inst >> 12) & 0x7) as u8;
    let rs1 = ((inst >> 15) & 0x1f) as u8;
    let rs2 = ((inst >> 20) & 0x1f) as u8;
    let funct7 = (inst >> 25) & 0x7f;

    let decoded = match opcode {
        0x03 => match funct3 {   // LOAD
            0x0 => Instruction::Lb { rd, rs1, imm: imm_i(inst) },
            0x1 => Instruction::Lh { rd, rs1, imm: imm_i(inst) },
            0x2 => Instruction::Lw { rd, rs1, imm: imm_i(inst) },
            0x3 => Instruction::Ld { rd, rs1, imm: imm_i(inst) },
            0x4 => Instruction::Lbu { rd, rs1, imm: imm_i(inst) },
            0x5 => Instruction::Lhu { rd, rs1, imm: imm_i(inst) },
            0x6 => Instruction::Lwu { rd, rs1, imm: imm_i(inst) },
            f => return decode_error(inst, address, format!("load funct3 {f}")),
        },
        0x13 => match funct3 {   // OP-IMM
            0x0 => Instruction::Addi { rd, rs1, imm: imm_i(inst) },
            0x2 => Instruction::Slti { rd, rs1, imm: imm_i(inst) },
            0x3 => Instruction::Sltiu { rd, rs1, imm: imm_i(inst) },
            0x4 => Instruction::Xori { rd, rs1, imm: imm_i(inst) },
            0x6 => Instruction::Ori { rd, rs1, imm: imm_i(inst) },
            0x7 => Instruction::Andi { rd, rs1, imm: imm_i(inst) },
            0x1 => Instruction::Slli { rd, rs1, shamt: shamt_i(inst) },
            0x5 => match funct7 {
                0x00 => Instruction::Srli { rd, rs1, shamt: shamt_i(inst) },
                0x20 => Instruction::Srai { rd, rs1, shamt: shamt_i(inst) },
                f => return decode_error(inst, address, format!("srli/srai funct7 {f}")),
            },
            f => return decode_error(inst, address, format!("op-imm funct3 {f}")),
        },
        0x17 => Instruction::Auipc { rd, imm: imm_u(inst) },          // AUIPC
        0x37 => Instruction::Lui { rd, imm: imm_u(inst) },             // LUI
        0x23 => match funct3 {                                         // STORE
            0x0 => Instruction::Sb { rs1, rs2, imm: imm_s(inst) },
            0x1 => Instruction::Sh { rs1, rs2, imm: imm_s(inst) },
            0x2 => Instruction::Sw { rs1, rs2, imm: imm_s(inst) },
            0x3 => Instruction::Sd { rs1, rs2, imm: imm_s(inst) },
            f => return decode_error(inst, address, format!("store funct3 {f}")),
        },
        0x33 => match funct3 {                                         // OP
            0x0 => match funct7 {
                0x00 => Instruction::Add { rd, rs1, rs2 },
                0x20 => Instruction::Sub { rd, rs1, rs2 },
                f => return decode_error(inst, address, format!("add/sub funct7 {f}")),
            },
            0x1 => Instruction::Sll { rd, rs1, rs2 },
            0x2 => Instruction::Slt { rd, rs1, rs2 },
            0x3 => Instruction::Sltu { rd, rs1, rs2 },
            0x4 => Instruction::Xor { rd, rs1, rs2 },
            0x5 => match funct7 {
                0x00 => Instruction::Srl { rd, rs1, rs2 },
                0x20 => Instruction::Sra { rd, rs1, rs2 },
                f => return decode_error(inst, address, format!("srl/sra funct7 {f}")),
            },
            0x6 => Instruction::Or { rd, rs1, rs2 },
            0x7 => Instruction::And { rd, rs1, rs2 },
            f => return decode_error(inst, address, format!("op funct3 {f}")),
        },
        0x63 => match funct3 {                                         // BRANCH
            0x0 => Instruction::Beq { rs1, rs2, imm: imm_b(inst) },
            0x1 => Instruction::Bne { rs1, rs2, imm: imm_b(inst) },
            0x4 => Instruction::Blt { rs1, rs2, imm: imm_b(inst) },
            0x5 => Instruction::Bge { rs1, rs2, imm: imm_b(inst) },
            0x6 => Instruction::Bltu { rs1, rs2, imm: imm_b(inst) },
            0x7 => Instruction::Bgeu { rs1, rs2, imm: imm_b(inst) },
            f => return decode_error(inst, address, format!("branch funct3 {f}")),
        },
        0x6f => Instruction::Jal { rd, imm: imm_j(inst) },             // JAL
        0x67 => Instruction::Jalr { rd, rs1, imm: imm_i(inst) },       // JALR
        0x73 => match funct3 {                                         // SYSTEM
            0x0 => match rs1 {
                0x0 => Instruction::Ecall,
                0x1 => Instruction::Ebreak,
                r => return decode_error(inst, address, format!("system rs1 {r}")),
            },
            0x1 => Instruction::Csrrw { rd, rs1, csr: imm_csr(inst) },
            0x2 => Instruction::Csrrs { rd, rs1, csr: imm_csr(inst) },
            0x3 => Instruction::Csrrc { rd, rs1, csr: imm_csr(inst) },
            0x5 => Instruction::Csrrwi { rd, zimm: imm_zimm(inst), csr: imm_csr(inst) },
            0x6 => Instruction::Csrrsi { rd, zimm: imm_zimm(inst), csr: imm_csr(inst) },
            0x7 => Instruction::Csrrci { rd, zimm: imm_zimm(inst), csr: imm_csr(inst) },
            f => return decode_error(inst, address, format!("system funct3 {f}")),
        },
        op => return decode_error(inst, address, format!("opcode 0x{op:02x}")),
    };

    Ok(Decoded {
        address,
        size: 4,
        instruction: decoded,
    })
}

fn decode_error(inst: u32, address: u64, detail: String) -> Result<Decoded> {
    Err(Error::Decode {
        opcode: inst,
        address,
        detail,
    })
}

// ---------- Immediate decoders (sign-extended where applicable) ----------

/// I-type immediate: bits [31:20], sign-extended to 12 bits.
fn imm_i(inst: u32) -> i64 {
    let imm = (inst >> 20) & 0xfff;
    sext(imm as u64, 12)
}

/// S-type immediate: imm[11:5] = bits[31:25], imm[4:0] = bits[11:7].
fn imm_s(inst: u32) -> i64 {
    let imm = ((inst & 0xfe000000) >> 20) | ((inst >> 7) & 0x1f);
    sext(imm as u64, 12)
}

/// B-type immediate: imm[12] = bit[31], imm[11] = bit[7], imm[10:5] = bits[30:25],
/// imm[4:1] = bits[11:8], imm[0] = 0. Sign-extended to 13 bits.
fn imm_b(inst: u32) -> i64 {
    let imm = ((inst & 0x80000000) >> 19)
        | ((inst & 0x80) << 4)
        | ((inst >> 20) & 0x7e0)
        | ((inst >> 7) & 0x1e);
    sext(imm as u64, 13)
}

/// U-type immediate: bits[31:12] << 12.
fn imm_u(inst: u32) -> i64 {
    (inst & 0xfffff000) as i64
}

/// J-type immediate: imm[20] = bit[31], imm[10:1] = bits[30:21],
/// imm[11] = bit[20], imm[19:12] = bits[19:12], imm[0] = 0. Sign-extended to 21 bits.
fn imm_j(inst: u32) -> i64 {
    let imm = ((inst & 0x80000000) >> 11)
        | ((inst & 0x00100000) >> 9)
        | ((inst & 0x7fe00000) >> 20)
        | ((inst & 0xff000) >> 12);
    sext(imm as u64, 21)
}

/// CSR register field (bits[31:20]).
fn imm_csr(inst: u32) -> u16 {
    ((inst >> 20) & 0xfff) as u16
}

/// Zimm field (bits[19:15]).
fn imm_zimm(inst: u32) -> u16 {
    ((inst >> 15) & 0x1f) as u16
}

/// Shift amount from I-type immediate (bits[24:20] for SLLI/SRLI/SRAI).
fn shamt_i(inst: u32) -> u64 {
    ((inst >> 20) & 0x1f) as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- Bit-field instruction packers (RISC-V ISA manual layout) ----------

    fn pack_i(opcode: u32, rd: u8, f3: u8, rs1: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0xfff) as u32;
        opcode
            | ((rd as u32) << 7)
            | ((f3 as u32) << 12)
            | ((rs1 as u32) << 15)
            | (imm_bits << 20)
    }

    fn pack_r(opcode: u32, f7: u32, f3: u8, rd: u8, rs1: u8, rs2: u8) -> u32 {
        opcode
            | (f7 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | ((rd as u32) << 7)
    }

    fn pack_s(opcode: u32, f3: u8, rs1: u8, rs2: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0xfff) as u32;
        let imm11_5 = (imm_bits >> 5) & 0x7f;
        let imm4_0 = imm_bits & 0x1f;
        opcode
            | (imm11_5 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | (imm4_0 << 7)
    }

    fn pack_b(opcode: u32, f3: u8, rs1: u8, rs2: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0x1fff) as u32;
        let imm12 = (imm_bits >> 12) & 1;
        let imm11 = (imm_bits >> 11) & 1;
        let imm10_5 = (imm_bits >> 5) & 0x3f;
        let imm4_1 = (imm_bits >> 1) & 0xf;
        opcode
            | (imm12 << 31)
            | (imm10_5 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | (imm11 << 7)
            | (imm4_1 << 8)
    }

    fn pack_u(opcode: u32, rd: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0xfffff000) as u32;
        opcode | imm_bits | ((rd as u32) << 7)
    }

    fn pack_j(opcode: u32, rd: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0x1fffff) as u32;
        let imm20 = (imm_bits >> 20) & 1;
        let imm19_12 = (imm_bits >> 12) & 0xff;
        let imm11 = (imm_bits >> 11) & 1;
        let imm10_1 = (imm_bits >> 1) & 0x3ff;
        opcode
            | (imm20 << 31)       // inst[31] = imm[20]
            | (imm19_12 << 12)    // inst[19:12] = imm[19:12]
            | (imm11 << 20)       // inst[20] = imm[11]
            | (imm10_1 << 21)     // inst[30:21] = imm[10:1]
            | ((rd as u32) << 7)
    }

    fn pack_system(opcode: u32, f3: u8, rd: u8, rs1: u8, csr: u16) -> u32 {
        opcode
            | ((csr as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | ((rd as u32) << 7)
    }

    fn dec(inst: u32, address: u64) -> Decoded {
        Decoder::new()
            .decode(&inst.to_le_bytes(), address)
            .unwrap()
    }

    #[test]
    fn addi() {
        let d = dec(pack_i(0x13, 1, 0, 2, 100), 0x1000);
        assert_eq!(d.address, 0x1000);
        assert_eq!(d.size, 4);
        assert!(matches!(d.instruction, Instruction::Addi { rd, rs1, imm }
            if rd == 1 && rs1 == 2 && imm == 100));
    }

    #[test]
    fn addi_negative_imm() {
        let d = dec(pack_i(0x13, 0, 0, 0, -1), 0);
        assert!(matches!(d.instruction, Instruction::Addi { imm, .. } if imm == -1));
    }

    #[test]
    fn add() {
        let d = dec(pack_r(0x33, 0x00, 0, 1, 2, 3), 0x2000);
        assert!(matches!(d.instruction, Instruction::Add { rd, rs1, rs2 }
            if rd == 1 && rs1 == 2 && rs2 == 3));
    }

    #[test]
    fn sub() {
        let d = dec(pack_r(0x33, 0x20, 0, 1, 2, 3), 0);
        assert!(matches!(d.instruction, Instruction::Sub { rd, rs1, rs2 }
            if rd == 1 && rs1 == 2 && rs2 == 3));
    }

    #[test]
    fn sll_srl_sra() {
        let d = dec(pack_r(0x33, 0x00, 1, 1, 2, 3), 0);
        assert!(matches!(d.instruction, Instruction::Sll { rd, rs1, rs2 }
            if rd == 1 && rs1 == 2 && rs2 == 3));
        let d = dec(pack_r(0x33, 0x00, 5, 1, 2, 3), 0);
        assert!(matches!(d.instruction, Instruction::Srl { rd, rs1, rs2 }
            if rd == 1 && rs1 == 2 && rs2 == 3));
        let d = dec(pack_r(0x33, 0x20, 5, 1, 2, 3), 0);
        assert!(matches!(d.instruction, Instruction::Sra { rd, rs1, rs2 }
            if rd == 1 && rs1 == 2 && rs2 == 3));
    }

    #[test]
    fn ld_sd() {
        let d = dec(pack_i(0x03, 1, 3, 2, 8), 0);
        assert!(matches!(d.instruction, Instruction::Ld { rd, rs1, imm }
            if rd == 1 && rs1 == 2 && imm == 8));
        let d = dec(pack_s(0x23, 3, 1, 2, 16), 0);
        assert!(matches!(d.instruction, Instruction::Sd { rs1, rs2, imm }
            if rs1 == 1 && rs2 == 2 && imm == 16));
    }

    #[test]
    fn beq_bne() {
        let d = dec(pack_b(0x63, 0, 1, 2, 4), 0x1000);
        assert!(matches!(d.instruction, Instruction::Beq { rs1, rs2, imm }
            if rs1 == 1 && rs2 == 2 && imm == 4));
        let d = dec(pack_b(0x63, 1, 1, 2, -8), 0);
        assert!(matches!(d.instruction, Instruction::Bne { rs1, rs2, imm }
            if rs1 == 1 && rs2 == 2 && imm == -8));
    }

    #[test]
    fn jal_jalr() {
        let d = dec(pack_j(0x6f, 1, 2048), 0);
        assert!(matches!(d.instruction, Instruction::Jal { rd, imm }
            if rd == 1 && imm == 2048));
        let d = dec(pack_j(0x6f, 0, 0), 0);
        assert!(matches!(d.instruction, Instruction::Jal { rd, imm }
            if rd == 0 && imm == 0));
        let d = dec(pack_i(0x67, 1, 0, 2, 0), 0);
        assert!(matches!(d.instruction, Instruction::Jalr { rd, rs1, imm }
            if rd == 1 && rs1 == 2 && imm == 0));
    }

    #[test]
    fn lui_auipc() {
        let d = dec(pack_u(0x37, 1, 0x12345000), 0);
        assert!(matches!(d.instruction, Instruction::Lui { rd, imm }
            if rd == 1 && imm == 0x12345000));
        let d = dec(pack_u(0x17, 1, 0), 0);
        assert!(matches!(d.instruction, Instruction::Auipc { rd, imm }
            if rd == 1 && imm == 0));
    }

    #[test]
    fn ecall_ebreak() {
        let d = dec(pack_system(0x73, 0, 0, 0, 0), 0);
        assert!(matches!(d.instruction, Instruction::Ecall));
        let d = dec(pack_system(0x73, 0, 0, 1, 0), 0);
        assert!(matches!(d.instruction, Instruction::Ebreak));
    }

    #[test]
    fn csr_instructions() {
        let d = dec(pack_system(0x73, 1, 1, 2, 0xc00), 0);
        assert!(matches!(d.instruction, Instruction::Csrrw { rd, rs1, csr }
            if rd == 1 && rs1 == 2 && csr == 0xc00));
        let d = dec(pack_system(0x73, 2, 1, 2, 0xc01), 0);
        assert!(matches!(d.instruction, Instruction::Csrrs { rd, rs1, csr }
            if rd == 1 && rs1 == 2 && csr == 0xc01));
        let d = dec(pack_system(0x73, 3, 1, 2, 0xc02), 0);
        assert!(matches!(d.instruction, Instruction::Csrrc { rd, rs1, csr }
            if rd == 1 && rs1 == 2 && csr == 0xc02));
        let d = dec(pack_system(0x73, 5, 1, 7, 0xc00), 0);
        assert!(matches!(d.instruction, Instruction::Csrrwi { rd, zimm, csr }
            if rd == 1 && zimm == 7 && csr == 0xc00));
    }

    #[test]
    fn slli_shamt() {
        let d = dec(pack_i(0x13, 1, 1, 2, 5), 0);
        assert!(matches!(d.instruction, Instruction::Slli { rd, rs1, shamt }
            if rd == 1 && rs1 == 2 && shamt == 5));
    }

    #[test]
    fn srai_shamt() {
        let d = dec(pack_i(0x13, 1, 5, 2, 31) | (0x20u32 << 25), 0);
        assert!(matches!(d.instruction, Instruction::Srai { rd, rs1, shamt }
            if rd == 1 && rs1 == 2 && shamt == 31));
    }

    #[test]
    fn slt_sltu_xor_or_and() {
        assert!(matches!(dec(pack_r(0x33, 0, 2, 1, 2, 3), 0).instruction,
            Instruction::Slt { .. }));
        assert!(matches!(dec(pack_r(0x33, 0, 3, 1, 2, 3), 0).instruction,
            Instruction::Sltu { .. }));
        assert!(matches!(dec(pack_r(0x33, 0, 4, 1, 2, 3), 0).instruction,
            Instruction::Xor { .. }));
        assert!(matches!(dec(pack_r(0x33, 0, 6, 1, 2, 3), 0).instruction,
            Instruction::Or { .. }));
        assert!(matches!(dec(pack_r(0x33, 0, 7, 1, 2, 3), 0).instruction,
            Instruction::And { .. }));
    }

    #[test]
    fn unknown_opcode() {
        let err = Decoder::new()
            .decode(&0x0000002fu32.to_le_bytes(), 0x1000)
            .unwrap_err();
        assert!(matches!(err, Error::Decode { .. }));
    }

    #[test]
    fn unknown_funct3_load() {
        let err = Decoder::new()
            .decode(&pack_i(0x03, 1, 7, 2, 0).to_le_bytes(), 0)
            .unwrap_err();
        assert!(matches!(err, Error::Decode { .. }));
    }

    #[test]
    fn truncated() {
        let err = Decoder::new().decode(&[0x13, 0x00], 0).unwrap_err();
        assert!(matches!(err, Error::Truncated { .. }));
    }
}
