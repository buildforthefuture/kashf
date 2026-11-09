// Copyright (c) <OWNER>

// L4 Humanizer: English language template. Original design.

use crate::humanizer::{Humanizer, Language};
use crate::ir::{IrEffect, IrExpr, IrFunction, IrNode};

/// English humanizer. Produces plain-language explanations, each tagged with
/// its source address for traceability.
pub struct EnglishHumanizer;

impl EnglishHumanizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EnglishHumanizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Humanizer for EnglishHumanizer {
    fn language(&self) -> Language {
        Language::En
    }

    fn format_node(&self, node: &IrNode) -> String {
        let addr = node.address;
        let body = match &node.effect {
            IrEffect::RegRead { reg } => {
                format!("Read register {} (address 0x{:x}).", reg_name(*reg), addr)
            }
            IrEffect::RegWrite { reg, value } => {
                format!(
                    "Write the value {} into register {} (address 0x{:x}).",
                    format_expr(value),
                    reg_name(*reg),
                    addr
                )
            }
            IrEffect::MemLoad { dst, address, size } => {
                format!(
                    "Load {} bytes from memory at {} into register {} (address 0x{:x}).",
                    size,
                    format_expr(address),
                    reg_name(*dst),
                    addr
                )
            }
            IrEffect::MemStore { address, value, size } => {
                format!(
                    "Store {} bytes of {} into memory at {} (address 0x{:x}).",
                    size,
                    format_expr(value),
                    format_expr(address),
                    addr
                )
            }
            IrEffect::Branch { condition, target } => {
                format!(
                    "If {}, branch to 0x{:x} (address 0x{:x}).",
                    format_expr(condition),
                    target,
                    addr
                )
            }
            IrEffect::Jump { target } => {
                format!("Jump to 0x{:x} (address 0x{:x}).", target, addr)
            }
            IrEffect::Call { target } => {
                format!("Call function at 0x{:x} (address 0x{:x}).", target, addr)
            }
            IrEffect::Return => {
                format!("Return from function (address 0x{:x}).", addr)
            }
            IrEffect::Syscall => {
                format!("Make a system call (address 0x{:x}).", addr)
            }
            IrEffect::Nop => {
                format!("No operation (address 0x{:x}).", addr)
            }
        };
        body
    }
}

fn reg_name(reg: u8) -> String {
    crate::decoder::tables::reg_name(reg).to_string()
}

fn format_expr(expr: &IrExpr) -> String {
    super::format_expr(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::lift_instruction;
    use crate::decoder::Decoder;

    // ---------- Self-contained bit-field packers ----------
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

    fn dec(inst: u32, address: u64) -> crate::decoder::Decoded {
        Decoder::new().decode(&inst.to_le_bytes(), address).unwrap()
    }

    #[test]
    fn en_regwrite() {
        let h = EnglishHumanizer::new();
        let d = dec(pack_i(0x13, 1, 0, 2, 100), 0x1000);
        let nodes = lift_instruction(&d);
        let s = h.format_node(&nodes[0]);
        assert!(s.contains("sp"));
        assert!(s.contains("100"));
        assert!(s.contains("ra"));
        assert!(s.contains("0x1000"));
    }

    #[test]
    fn en_branch() {
        let h = EnglishHumanizer::new();
        let d = dec(pack_b(0x63, 0, 1, 2, 4), 0x1000);
        let nodes = lift_instruction(&d);
        let s = h.format_node(&nodes[0]);
        assert!(s.contains("branch to 0x1004"));
        assert!(s.contains("0x1000"));
    }

    #[test]
    fn en_jump() {
        let h = EnglishHumanizer::new();
        let d = dec(pack_j(0x6f, 1, 2048), 0x1000);
        let nodes = lift_instruction(&d);
        let s = h.format_node(&nodes[1]); // the Jump node
        assert!(s.contains("Jump to 0x1800"));
        assert!(s.contains("0x1000"));
    }

    #[test]
    fn en_syscall() {
        let h = EnglishHumanizer::new();
        let d = dec(0x00000073, 0x4000); // ecall
        let nodes = lift_instruction(&d);
        let s = h.format_node(&nodes[0]);
        assert!(s.contains("system call"));
        assert!(s.contains("0x4000"));
    }

    #[test]
    fn en_function() {
        let h = EnglishHumanizer::new();
        let d = dec(pack_i(0x13, 1, 0, 2, 100), 0x1000);
        let nodes = lift_instruction(&d);
        let func = crate::ir::IrFunction {
            name: "main".to_string(),
            address: 0x1000,
            nodes,
        };
        let s = h.format_function(&func);
        assert!(s.contains("Function main at 0x1000:"));
        assert!(s.contains("0x1000"));
    }
}
