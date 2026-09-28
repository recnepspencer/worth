//! Static cost formulas of the `worth-expression-cost-v1` contract.
//!
//! Every node charges one work unit when it is visited. An operation whose
//! work does not depend on its operands' contents also charges a fixed extra
//! amount before it runs: decimal arithmetic a constant for its bounded
//! 38-digit intermediates, widened tolerance and unit or decimal conversions a
//! constant for their bounded wide integers, and bus operations one unit per
//! 64-bit limb of every bus operand and result plane. Work that depends on
//! contents (comparisons, text scans, collection traversal, and copies) is
//! charged by the operation as it happens.

use crate::expressions::functions::Builtin;
use crate::expressions::program::{ExpressionProgram, Op, ProgramNode};
use crate::expressions::syntax::ast::BinaryOp;
use crate::expressions::types::ExpressionType;

/// Decimal `+`, `-`, `*`, `decimal_div`, `quantize`, and one `sum` step.
pub(super) const DECIMAL_WORK: u64 = 8;

/// `near`, `quantity`, `magnitude`, and casts to or from Decimal.
pub(super) const WIDE_WORK: u64 = 64;

/// Limbs across a bus type's planes; zero for other types.
pub(super) fn planes(ty: &ExpressionType) -> u64 {
    match ty {
        ExpressionType::Bits(width) => u64::from(width.div_ceil(64)),
        ExpressionType::Logic4(width) => u64::from(width.div_ceil(64)) * 2,
        _ => 0,
    }
}

fn is_bus_operation(builtin: Builtin) -> bool {
    matches!(
        builtin,
        Builtin::BitAnd
            | Builtin::BitOr
            | Builtin::BitXor
            | Builtin::BitNot
            | Builtin::ShiftLeft
            | Builtin::ShiftRight
            | Builtin::Concat
            | Builtin::BitsAdd
            | Builtin::WrappingAdd
            | Builtin::LogicEq
            | Builtin::CaseEqual
            | Builtin::Mux
            | Builtin::Truncate
            | Builtin::Extend
            | Builtin::ToBits
            | Builtin::BitSlice { .. }
            | Builtin::ExactCast
    )
}

/// The extra work `node` charges before it runs.
pub(super) fn static_cost(program: &ExpressionProgram, node: &ProgramNode) -> u64 {
    let types = || {
        node.children
            .iter()
            .map(|child| &program.nodes()[*child as usize].ty)
            .chain([&node.ty])
    };
    let decimal = types().any(|ty| *ty == ExpressionType::Decimal);
    match &node.op {
        Op::Binary(BinaryOp::Add | BinaryOp::Subtract | BinaryOp::Multiply) if decimal => {
            DECIMAL_WORK
        }
        Op::Builtin(Builtin::DecimalDiv | Builtin::Quantize) => DECIMAL_WORK,
        Op::Builtin(Builtin::Near | Builtin::Quantity(_) | Builtin::Magnitude(_)) => WIDE_WORK,
        Op::Builtin(Builtin::ExactCast | Builtin::RoundedCast) if decimal => WIDE_WORK,
        Op::Builtin(builtin) if is_bus_operation(*builtin) => types().map(planes).sum(),
        _ => 0,
    }
}

/// Started 8-byte words in `bytes`: the work of inspecting, hashing, or
/// copying them.
pub(super) fn words(bytes: u64) -> u64 {
    bytes.div_ceil(8)
}
