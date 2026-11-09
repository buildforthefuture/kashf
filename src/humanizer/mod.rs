// Copyright (c) <OWNER>

// L4 Humanizer: IR to plain-language text. Swappable language templates
// (en first, then ar). Every sentence links back to the source address for
// traceability. Clean-room, original design.

use crate::ir::{IrEffect, IrExpr, IrFunction, IrNode};

pub mod en;

/// Supported humanization languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    En,
    Ar,
}

/// Humanizes IR nodes into plain-language explanations.
pub trait Humanizer {
    fn language(&self) -> Language;

    /// Format a single IR node into one or more plain-language sentences.
    /// Every sentence references the node's source address for traceability.
    fn format_node(&self, node: &IrNode) -> String;

    /// Format an entire function, one line per IR node.
    fn format_function(&self, func: &IrFunction) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "Function {} at 0x{:x}:\n",
            func.name, func.address
        ));
        for node in &func.nodes {
            out.push_str(&format!("  {}\n", self.format_node(node)));
        }
        out
    }
}

/// Format an IrExpr into a plain-language fragment.
fn format_expr(expr: &IrExpr) -> String {
    match expr {
        IrExpr::Const { value, .. } => {
            if *value >= 0x1000 && *value < 0x1_0000_0000 {
                format!("0x{:x}", value)
            } else {
                value.to_string()
            }
        }
        IrExpr::Reg { reg, .. } => crate::decoder::tables::reg_name(*reg).to_string(),
        IrExpr::Mem { ptr, size, .. } => {
            format!("{} bytes at {}", size, format_expr(ptr))
        }
        IrExpr::Add { lhs, rhs, .. } => {
            format!("({} + {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Sub { lhs, rhs, .. } => {
            format!("({} - {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::And { lhs, rhs, .. } => {
            format!("({} AND {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Or { lhs, rhs, .. } => {
            format!("({} OR {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Xor { lhs, rhs, .. } => {
            format!("({} XOR {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Shl { lhs, rhs, .. } => {
            format!("({} << {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Shr { lhs, rhs, .. } => {
            format!("({} >> {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Sra { lhs, rhs, .. } => {
            format!("({} arithmetic>> {})", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Eq { lhs, rhs, .. } => {
            format!("{} equals {}", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Ne { lhs, rhs, .. } => {
            format!("{} does not equal {}", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Lt { lhs, rhs, .. } => {
            format!("{} is less than {}", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Ltu { lhs, rhs, .. } => {
            format!("{} is less than (unsigned) {}", format_expr(lhs), format_expr(rhs))
        }
        IrExpr::Neg { operand, .. } => {
            format!("negate {}", format_expr(operand))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_expr_reg_and_const() {
        let e = IrExpr::Reg { reg: 2, address: 0 };
        assert_eq!(format_expr(&e), "sp");
        let e = IrExpr::Const { value: 100, address: 0 };
        assert_eq!(format_expr(&e), "100");
        let e = IrExpr::Const { value: 0x1000, address: 0 };
        assert_eq!(format_expr(&e), "0x1000");
    }
}
