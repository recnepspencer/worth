//! The admitted typed program: canonical post-order nodes with resolved slots,
//! binders, functions, and literal values.

use crate::expressions::functions::Builtin;
use crate::expressions::syntax::ast::{BinaryOp, ComprehensionKind, UnaryOp};
use crate::expressions::syntax::SourceOrigin;
use crate::expressions::types::ExpressionType;

/// An exact admitted literal. Its type is the node type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Literal {
    Bool(bool),
    Integer(i128),
    Float32(u32),
    Float64(u64),
    /// `coefficient * 10^-scale`, normalized: no redundant trailing zeros.
    Decimal {
        coefficient: i128,
        scale: u8,
    },
    String(Box<str>),
    Bytes(Box<[u8]>),
    Enum(u32),
    /// Canonical SI magnitude bits.
    Quantity(u64),
    /// Little-endian 64-bit limbs; bit 0 is the least significant bit.
    Bits(Box<[u64]>),
    /// Two bitplanes: `(value, unknown)` is 0=(0,0), 1=(1,0), X=(0,1), Z=(1,1).
    Logic4 {
        value: Box<[u64]>,
        unknown: Box<[u64]>,
    },
    None,
}

/// A typed program operation. Children are listed on the node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum Op {
    Literal(Literal),
    /// Operand slot, in canonical order over the operands the program uses.
    Operand(u32),
    /// De Bruijn index: 0 is the innermost `let` or comprehension binder.
    Local(u32),
    /// Field index in schema order (`MapEntry`: 0 key, 1 value).
    Field(u32),
    Unary(UnaryOp),
    Binary(BinaryOp),
    Conditional,
    /// Children `[value, body]`; the body sees the value as `Local(0)`.
    Let,
    /// Children `[base, body]`; the body sees each element as `Local(0)`.
    Comprehension(ComprehensionKind),
    Builtin(Builtin),
    /// Index into the program's installed-function closure.
    Call(u32),
    List,
    /// Children alternate key, value in authored order.
    Map,
    /// Children are field values in schema order.
    Record,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProgramNode {
    pub(crate) op: Op,
    pub(crate) ty: ExpressionType,
    pub(crate) children: Box<[u32]>,
    pub(crate) origin: SourceOrigin,
}

/// Nodes in canonical post-order: children in evaluation order precede their
/// parent, and the root is last.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExpressionProgram {
    nodes: Box<[ProgramNode]>,
}

impl ExpressionProgram {
    pub(crate) fn new(nodes: Vec<ProgramNode>) -> Self {
        debug_assert!(!nodes.is_empty(), "programs have a root");
        Self {
            nodes: nodes.into_boxed_slice(),
        }
    }

    pub(crate) fn nodes(&self) -> &[ProgramNode] {
        &self.nodes
    }

    pub(crate) fn result_type(&self) -> &ExpressionType {
        &self.nodes[self.nodes.len() - 1].ty
    }
}
