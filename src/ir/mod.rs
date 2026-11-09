// Copyright (c) <OWNER>

// L3 IR: a small intermediate representation with explicit effects.
// Every IR node and expression keeps its source address for traceability.
// Clean-room, original design. No unsafe.

use crate::decoder::{Decoded, Instruction};

/// An expression in the IR. Every variant records the source address of the
/// instruction it was lifted from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrExpr {
    Const { value: i64, address: u64 },
    Reg { reg: u8, address: u64 },
    /// Memory reference: load/store of `size` bytes at `ptr`.
    Mem {
        ptr: Box<IrExpr>,
        size: u8,
        address: u64,
    },
    Add {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Sub {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    And {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Or {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Xor {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Shl {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Shr {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Sra {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Eq {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Ne {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Lt {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Ltu {
        lhs: Box<IrExpr>,
        rhs: Box<IrExpr>,
        address: u64,
    },
    Neg { operand: Box<IrExpr>, address: u64 },
}

impl IrExpr {
    pub fn address(&self) -> u64 {
        match self {
            IrExpr::Const { address, .. }
            | IrExpr::Reg { address, .. }
            | IrExpr::Mem { address, .. }
            | IrExpr::Add { address, .. }
            | IrExpr::Sub { address, .. }
            | IrExpr::And { address, .. }
            | IrExpr::Or { address, .. }
            | IrExpr::Xor { address, .. }
            | IrExpr::Shl { address, .. }
            | IrExpr::Shr { address, .. }
            | IrExpr::Sra { address, .. }
            | IrExpr::Eq { address, .. }
            | IrExpr::Ne { address, .. }
            | IrExpr::Lt { address, .. }
            | IrExpr::Ltu { address, .. }
            | IrExpr::Neg { address, .. } => *address,
        }
    }
}

/// A single effectful IR node. Every node carries the source address of the
/// instruction it was lifted from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrEffect {
    /// Read a register (produces a value used later).
    RegRead { reg: u8 },
    /// Write a register.
    RegWrite { reg: u8, value: IrExpr },
    /// Load `size` bytes from memory into register `dst`.
    MemLoad { dst: u8, address: IrExpr, size: u8 },
    /// Store `size` bytes of `value` to memory at `address`.
    MemStore { address: IrExpr, value: IrExpr, size: u8 },
    /// Conditional branch to `target` when `condition` is true.
    Branch { condition: IrExpr, target: u64 },
    /// Unconditional jump to `target`.
    Jump { target: u64 },
    /// Function call to `target`.
    Call { target: u64 },
    /// Function return.
    Return,
    /// System call.
    Syscall,
    /// No operation.
    Nop,
}

/// A single IR instruction: an effect at a source address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrNode {
    pub address: u64,
    pub effect: IrEffect,
}

impl IrNode {
    pub fn new(address: u64, effect: IrEffect) -> Self {
        Self { address, effect }
    }
}

/// A lifted function: name, entry address, and its IR node stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrFunction {
    pub name: String,
    pub address: u64,
    pub nodes: Vec<IrNode>,
}

impl IrFunction {
    pub fn new(name: String, address: u64) -> Self {
        Self {
            name,
            address,
            nodes: Vec::new(),
        }
    }
    pub fn push(&mut self, node: IrNode) {
        self.nodes.push(node);
    }
}

/// Lift a slice of decoded instructions into a list of IR nodes. One decoded
/// instruction may produce multiple IR nodes (e.g. a call writes the return
/// address register and then jumps).
pub fn lift_instruction(decoded: &Decoded) -> Vec<IrNode> {
    let addr = decoded.address;
    match &decoded.instruction {
        Instruction::Lb { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Lh { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 2,
                },
            )]
        }
        Instruction::Lw { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 4,
                },
            )]
        }
        Instruction::Ld { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 8,
                },
            )]
        }
        Instruction::Lbu { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 1,
                },
            )]
        }
        Instruction::Lhu { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 2,
                },
            )]
        }
        Instruction::Lwu { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemLoad {
                    dst: *rd,
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    size: 4,
                },
            )]
        }
        Instruction::Addi { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Slti { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Lt {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sltiu { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Ltu {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Xori { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Xor {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Ori { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Or {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Andi { rd, rs1, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::And {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Slli { rd, rs1, shamt } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Shl {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *shamt as i64,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Srli { rd, rs1, shamt } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Shr {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *shamt as i64,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Srai { rd, rs1, shamt } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Sra {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *shamt as i64,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Add { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sub { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Sub {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sll { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Shl {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Slt { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Lt {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sltu { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Ltu {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Xor { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Xor {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Srl { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Shr {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sra { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Sra {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Or { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Or {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::And { rd, rs1, rs2 } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::And {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Sb { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemStore {
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    value: IrExpr::Reg {
                        reg: *rs2,
                        address: addr,
                    },
                    size: 1,
                },
            )]
        }
        Instruction::Sh { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemStore {
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    value: IrExpr::Reg {
                        reg: *rs2,
                        address: addr,
                    },
                    size: 2,
                },
            )]
        }
        Instruction::Sw { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemStore {
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    value: IrExpr::Reg {
                        reg: *rs2,
                        address: addr,
                    },
                    size: 4,
                },
            )]
        }
        Instruction::Sd { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::MemStore {
                    address: IrExpr::Add {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                    value: IrExpr::Reg {
                        reg: *rs2,
                        address: addr,
                    },
                    size: 8,
                },
            )]
        }
        Instruction::Beq { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Eq {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Bne { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Ne {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Blt { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Lt {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Bge { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Ltu {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Bltu { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Ltu {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Bgeu { rs1, rs2, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::Branch {
                    condition: IrExpr::Ltu {
                        lhs: Box::new(IrExpr::Reg {
                            reg: *rs2,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Reg {
                            reg: *rs1,
                            address: addr,
                        }),
                        address: addr,
                    },
                    target: addr.wrapping_add(*imm as u64),
                },
            )]
        }
        Instruction::Jal { rd, imm } => {
            let mut nodes = Vec::new();
            if *rd != 0 {
                nodes.push(IrNode::new(
                    addr,
                    IrEffect::RegWrite {
                        reg: *rd,
                        value: IrExpr::Const {
                            value: (addr as i64).wrapping_add(4),
                            address: addr,
                        },
                    },
                ));
            }
            nodes.push(IrNode::new(
                addr,
                IrEffect::Jump {
                    target: addr.wrapping_add(*imm as u64),
                },
            ));
            nodes
        }
        Instruction::Jalr { rd, rs1, imm } => {
            let mut nodes = Vec::new();
            if *rd != 0 {
                nodes.push(IrNode::new(
                    addr,
                    IrEffect::RegWrite {
                        reg: *rd,
                        value: IrExpr::Const {
                            value: (addr as i64).wrapping_add(4),
                            address: addr,
                        },
                    },
                ));
            }
            nodes.push(IrNode::new(
                addr,
                IrEffect::Jump {
                    target: 0, // resolved at analysis time from rs1 + imm
                },
            ));
            nodes
        }
        Instruction::Auipc { rd, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Add {
                        lhs: Box::new(IrExpr::Const {
                            value: addr as i64,
                            address: addr,
                        }),
                        rhs: Box::new(IrExpr::Const {
                            value: *imm,
                            address: addr,
                        }),
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Lui { rd, imm } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Const {
                        value: *imm,
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Ecall => {
            vec![IrNode::new(addr, IrEffect::Syscall)]
        }
        Instruction::Ebreak => {
            vec![IrNode::new(addr, IrEffect::Nop)]
        }
        Instruction::Csrrw { rd, rs1, csr: _ } => {
            // CSR read/write: simplified as reg write from csr.
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Reg {
                        reg: *rs1,
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Csrrs { rd, rs1, csr: _ } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Reg {
                        reg: *rs1,
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Csrrc { rd, rs1, csr: _ } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Reg {
                        reg: *rs1,
                        address: addr,
                    },
                },
            )]
        }
        Instruction::Csrrwi { rd, zimm: _, csr: _ } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Const { value: 0, address: addr },
                },
            )]
        }
        Instruction::Csrrsi { rd, zimm: _, csr: _ } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Const { value: 0, address: addr },
                },
            )]
        }
        Instruction::Csrrci { rd, zimm: _, csr: _ } => {
            vec![IrNode::new(
                addr,
                IrEffect::RegWrite {
                    reg: *rd,
                    value: IrExpr::Const { value: 0, address: addr },
                },
            )]
        }
        Instruction::Unknown { opcode } => {
            vec![IrNode::new(
                addr,
                IrEffect::Nop,
            )]
        }
    }
}

/// Lift a sequence of decoded instructions (all at the same base) into one
/// IrFunction.
pub fn lift_function(name: String, address: u64, decoded: &[Decoded]) -> IrFunction {
    let mut func = IrFunction::new(name, address);
    for d in decoded {
        for node in lift_instruction(d) {
            func.push(node);
        }
    }
    func
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------- Bit-field instruction packers (RISC-V ISA manual layout) ----------
    // Self-contained copies mirroring decoder::riscv64 test helpers.

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
    fn pack_r(opcode: u32, f7: u32, f3: u8, rd: u8, rs1: u8, rs2: u8) -> u32 {
        opcode
            | (f7 << 25)
            | ((rs2 as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | ((rd as u32) << 7)
    }
    fn dec_inst(inst: u32, address: u64) -> Decoded {
        crate::decoder::Decoder::new()
            .decode(&inst.to_le_bytes(), address)
            .unwrap()
    }

    #[test]
    fn lift_addi() {
        let d = dec_inst(pack_i(0x13, 1, 0, 2, 100), 0x1000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].address, 0x1000);
        assert!(matches!(nodes[0].effect, IrEffect::RegWrite { reg, .. } if reg == 1));
    }

    #[test]
    fn lift_ld() {
        let d = dec_inst(pack_i(0x03, 1, 3, 2, 8), 0x2000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].effect, IrEffect::MemLoad { dst, size, .. } if dst == 1 && size == 8));
        assert_eq!(nodes[0].address, 0x2000);
    }

    #[test]
    fn lift_sd() {
        let d = dec_inst(pack_s_test(0x23, 3, 1, 2, 16), 0x3000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].effect, IrEffect::MemStore { size, .. } if size == 8));
    }

    #[test]
    fn lift_beq_branch_target() {
        let d = dec_inst(pack_b(0x63, 0, 1, 2, 4), 0x1000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].effect, IrEffect::Branch { target, .. } if target == 0x1004));
    }

    #[test]
    fn lift_jal_writes_ra_and_jumps() {
        let d = dec_inst(pack_j(0x6f, 1, 2048), 0x1000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 2);
        assert!(matches!(nodes[0].effect, IrEffect::RegWrite { reg, .. } if reg == 1));
        assert!(matches!(nodes[1].effect, IrEffect::Jump { target } if target == 0x1800));
    }

    #[test]
    fn lift_ecall() {
        let d = dec_inst(pack_system_test(0x73, 0, 0, 0, 0), 0x4000);
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].effect, IrEffect::Syscall));
    }

    #[test]
    fn lift_unknown_is_nop() {
        let d = Decoded {
            address: 0x5000,
            size: 4,
            instruction: Instruction::Unknown { opcode: 0xdeadbeef },
        };
        let nodes = lift_instruction(&d);
        assert_eq!(nodes.len(), 1);
        assert!(matches!(nodes[0].effect, IrEffect::Nop));
    }

    #[test]
    fn every_node_has_address() {
        let d = dec_inst(pack_r(0x33, 0x00, 0, 1, 2, 3), 0x9000);
        let nodes = lift_instruction(&d);
        for n in &nodes {
            assert_eq!(n.address, 0x9000);
        }
    }

    #[test]
    fn lift_function_empty() {
        let func = lift_function("main".into(), 0x1000, &[]);
        assert_eq!(func.name, "main");
        assert_eq!(func.address, 0x1000);
        assert!(func.nodes.is_empty());
    }

    // Re-declare small packer helpers (mirrors decoder tests) to keep this
    // module self-contained.
    fn pack_s_test(opcode: u32, f3: u8, rs1: u8, rs2: u8, imm: i64) -> u32 {
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
    fn pack_system_test(opcode: u32, f3: u8, rd: u8, rs1: u8, csr: u16) -> u32 {
        opcode
            | ((csr as u32) << 20)
            | ((rs1 as u32) << 15)
            | ((f3 as u32) << 12)
            | ((rd as u32) << 7)
    }
}
