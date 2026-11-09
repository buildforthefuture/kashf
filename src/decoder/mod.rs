// Copyright (c) <OWNER>

// L2 Decoder: RV64I public types, entry point, and text disassembler.
// Implementation lives in riscv64.rs; tables live in tables.rs.

pub mod riscv64;
pub mod tables;

pub use riscv64::{Decoded, Instruction, Decoder};
pub use tables::{reg_name, csr_name, sext};

use crate::error::Result;

/// Format a decoded instruction as a plain text disassembly line.
///
/// The output is deterministic: registers use ABI names, immediates are
/// printed as signed decimal, and branch/jump targets are shown as absolute
/// virtual addresses.
pub fn format_instruction(decoded: &Decoded) -> String {
    let addr = decoded.address;
    match &decoded.instruction {
        Instruction::Lb { rd, rs1, imm } => {
            format!("lb {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Lh { rd, rs1, imm } => {
            format!("lh {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Lw { rd, rs1, imm } => {
            format!("lw {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Ld { rd, rs1, imm } => {
            format!("ld {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Lbu { rd, rs1, imm } => {
            format!("lbu {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Lhu { rd, rs1, imm } => {
            format!("lhu {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Lwu { rd, rs1, imm } => {
            format!("lwu {}, {}({})", reg_name(*rd), imm, reg_name(*rs1))
        }
        Instruction::Addi { rd, rs1, imm } => {
            format!("addi {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Slti { rd, rs1, imm } => {
            format!("slti {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Sltiu { rd, rs1, imm } => {
            format!("sltiu {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Xori { rd, rs1, imm } => {
            format!("xori {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Ori { rd, rs1, imm } => {
            format!("ori {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Andi { rd, rs1, imm } => {
            format!("andi {}, {}, {}", reg_name(*rd), reg_name(*rs1), imm)
        }
        Instruction::Slli { rd, rs1, shamt } => {
            format!("slli {}, {}, {}", reg_name(*rd), reg_name(*rs1), shamt)
        }
        Instruction::Srli { rd, rs1, shamt } => {
            format!("srli {}, {}, {}", reg_name(*rd), reg_name(*rs1), shamt)
        }
        Instruction::Srai { rd, rs1, shamt } => {
            format!("srai {}, {}, {}", reg_name(*rd), reg_name(*rs1), shamt)
        }
        Instruction::Add { rd, rs1, rs2 } => {
            format!("add {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Sub { rd, rs1, rs2 } => {
            format!("sub {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Sll { rd, rs1, rs2 } => {
            format!("sll {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Slt { rd, rs1, rs2 } => {
            format!("slt {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Sltu { rd, rs1, rs2 } => {
            format!("sltu {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Xor { rd, rs1, rs2 } => {
            format!("xor {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Srl { rd, rs1, rs2 } => {
            format!("srl {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Sra { rd, rs1, rs2 } => {
            format!("sra {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Or { rd, rs1, rs2 } => {
            format!("or {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::And { rd, rs1, rs2 } => {
            format!("and {}, {}, {}", reg_name(*rd), reg_name(*rs1), reg_name(*rs2))
        }
        Instruction::Sb { rs1, rs2, imm } => {
            format!("sb {}, {}({})", reg_name(*rs2), imm, reg_name(*rs1))
        }
        Instruction::Sh { rs1, rs2, imm } => {
            format!("sh {}, {}({})", reg_name(*rs2), imm, reg_name(*rs1))
        }
        Instruction::Sw { rs1, rs2, imm } => {
            format!("sw {}, {}({})", reg_name(*rs2), imm, reg_name(*rs1))
        }
        Instruction::Sd { rs1, rs2, imm } => {
            format!("sd {}, {}({})", reg_name(*rs2), imm, reg_name(*rs1))
        }
        Instruction::Beq { rs1, rs2, imm } => {
            format!(
                "beq {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Bne { rs1, rs2, imm } => {
            format!(
                "bne {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Blt { rs1, rs2, imm } => {
            format!(
                "blt {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Bge { rs1, rs2, imm } => {
            format!(
                "bge {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Bltu { rs1, rs2, imm } => {
            format!(
                "bltu {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Bgeu { rs1, rs2, imm } => {
            format!(
                "bgeu {}, {}, 0x{:x}",
                reg_name(*rs1),
                reg_name(*rs2),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Jal { rd, imm } => {
            format!(
                "jal {}, 0x{:x}",
                reg_name(*rd),
                addr.wrapping_add(*imm as u64)
            )
        }
        Instruction::Jalr { rd, rs1, imm } => {
            format!(
                "jalr {}, {}({})",
                reg_name(*rd),
                imm,
                reg_name(*rs1)
            )
        }
        Instruction::Auipc { rd, imm } => {
            format!("auipc {}, {}", reg_name(*rd), imm)
        }
        Instruction::Lui { rd, imm } => {
            format!("lui {}, {}", reg_name(*rd), imm)
        }
        Instruction::Ecall => "ecall".to_string(),
        Instruction::Ebreak => "ebreak".to_string(),
        Instruction::Csrrw { rd, rs1, csr } => {
            format!("csrrw {}, {}, {}", reg_name(*rd), reg_name(*rs1), csr_name(*csr))
        }
        Instruction::Csrrs { rd, rs1, csr } => {
            format!("csrrs {}, {}, {}", reg_name(*rd), reg_name(*rs1), csr_name(*csr))
        }
        Instruction::Csrrc { rd, rs1, csr } => {
            format!("csrrc {}, {}, {}", reg_name(*rd), reg_name(*rs1), csr_name(*csr))
        }
        Instruction::Csrrwi { rd, zimm, csr } => {
            format!("csrrwi {}, {}, {}", reg_name(*rd), zimm, csr_name(*csr))
        }
        Instruction::Csrrsi { rd, zimm, csr } => {
            format!("csrrsi {}, {}, {}", reg_name(*rd), zimm, csr_name(*csr))
        }
        Instruction::Csrrci { rd, zimm, csr } => {
            format!("csrrci {}, {}, {}", reg_name(*rd), zimm, csr_name(*csr))
        }
        Instruction::Unknown { opcode } => {
            format!(".word 0x{:08x}", opcode)
        }
    }
}

/// Convenience: decode one instruction and format it.
pub fn disassemble_one(bytes: &[u8], address: u64) -> Result<(Decoded, String)> {
    let decoded = Decoder::new().decode(bytes, address)?;
    let text = format_instruction(&decoded);
    Ok((decoded, text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_addi() {
        let d = dec(pack_i(0x13, 1, 0, 2, 100), 0x1000);
        let s = format_instruction(&d);
        assert_eq!(s, "addi ra, sp, 100");
    }

    #[test]
    fn format_branch_target() {
        let d = dec(pack_b(0x63, 0, 1, 2, 4), 0x1000);
        let s = format_instruction(&d);
        assert_eq!(s, "beq ra, sp, 0x1004");
    }

    #[test]
    fn format_jal() {
        let d = dec(pack_j(0x6f, 1, 2048), 0x1000);
        let s = format_instruction(&d);
        assert_eq!(s, "jal ra, 0x1800");
    }

    #[test]
    fn format_lui() {
        let d = dec(pack_u(0x37, 1, 0x12345000), 0);
        let s = format_instruction(&d);
        assert_eq!(s, "lui ra, 305418240");
    }

    #[test]
    fn format_unknown() {
        let d = Decoded {
            address: 0,
            size: 4,
            instruction: Instruction::Unknown { opcode: 0xdeadbeef },
        };
        let s = format_instruction(&d);
        assert_eq!(s, ".word 0xdeadbeef");
    }

    #[test]
    fn disassemble_one_roundtrip() {
        let bytes = pack_i(0x13, 1, 0, 2, 100).to_le_bytes();
        let (d, s) = disassemble_one(&bytes, 0x1000).unwrap();
        assert_eq!(d.address, 0x1000);
        assert_eq!(s, "addi ra, sp, 100");
    }

    // Reuse the packer helpers from riscv64's tests by re-declaring minimal
    // copies here to keep this module self-contained.
    fn pack_i(opcode: u32, rd: u8, f3: u8, rs1: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0xfff) as u32;
        opcode | ((rd as u32) << 7) | ((f3 as u32) << 12) | ((rs1 as u32) << 15) | (imm_bits << 20)
    }
    fn pack_b(opcode: u32, f3: u8, rs1: u8, rs2: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0x1fff) as u32;
        opcode
            | (((imm_bits >> 12) & 1) << 31)
            | (((imm_bits >> 5) & 0x3f) << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | (((imm_bits >> 11) & 1) << 7)
            | (((imm_bits >> 1) & 0xf) << 8)
    }
    fn pack_j(opcode: u32, rd: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0x1fffff) as u32;
        opcode
            | (((imm_bits >> 20) & 1) << 31)
            | (((imm_bits >> 12) & 0xff) << 12)
            | (((imm_bits >> 11) & 1) << 20)
            | (((imm_bits >> 1) & 0x3ff) << 21)
            | ((rd as u32) << 7)
    }
    fn pack_u(opcode: u32, rd: u8, imm: i64) -> u32 {
        let imm_bits = (imm as u64 & 0xfffff000) as u32;
        opcode | imm_bits | ((rd as u32) << 7)
    }
    fn dec(inst: u32, address: u64) -> Decoded {
        Decoder::new().decode(&inst.to_le_bytes(), address).unwrap()
    }
}
