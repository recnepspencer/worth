//! Operator typing. No implicit promotion: operands share one type, except
//! that quantities scale by plain Float64 values.

use crate::expressions::denial::{ExpressionDenialDetail, ExpressionResult};
use crate::expressions::program::{Literal, Op};
use crate::expressions::syntax::ast::{BinaryOp, NodeId, SyntaxNode, UnaryOp};
use crate::expressions::types::ExpressionType;

use super::Checker;

impl Checker<'_> {
    pub(super) fn unary(
        &mut self,
        id: NodeId,
        op: UnaryOp,
        operand: NodeId,
    ) -> ExpressionResult<u32> {
        if op == UnaryOp::Negate {
            if let Some(folded) = self.negated_literal(id, operand)? {
                return Ok(folded);
            }
        }
        let expected = (op == UnaryOp::Not).then_some(ExpressionType::Bool);
        let operand = self.check(operand, expected.as_ref())?;
        let ty = self.ty(operand).clone();
        let negatable = matches!(&ty, ExpressionType::Integer(integer) if integer.is_signed())
            || matches!(
                ty,
                ExpressionType::Float32
                    | ExpressionType::Float64
                    | ExpressionType::Decimal
                    | ExpressionType::Quantity(_)
            );
        if op == UnaryOp::Negate && !negatable {
            return Err(self.expected_kind(id, "a signed numeric type", operand));
        }
        self.emit(id, Op::Unary(op), ty, vec![operand])
    }

    /// Folds `-literal` into one signed literal, so `-9223372036854775808`
    /// admits without overflowing its positive intermediate.
    fn negated_literal(&mut self, id: NodeId, operand: NodeId) -> ExpressionResult<Option<u32>> {
        match self.tree.node(operand) {
            SyntaxNode::Integer(magnitude) => {
                let value = i128::try_from(*magnitude)
                    .ok()
                    .map(|magnitude| -magnitude)
                    .filter(|value| *value >= i128::from(i64::MIN))
                    .ok_or_else(|| {
                        self.deny(
                            id,
                            ExpressionDenialDetail::InvalidValue("Int64 literal out of range"),
                        )
                    })?;
                let literal = self.literal(id, Literal::Integer(value), ExpressionType::INT64)?;
                Ok(Some(literal))
            }
            SyntaxNode::Float(decimal) => {
                let value = decimal.to_f64().ok_or_else(|| {
                    self.deny(
                        id,
                        ExpressionDenialDetail::InvalidValue("float literal is not finite"),
                    )
                })?;
                let bits = (-value + 0.0).to_bits();
                let literal = self.literal(id, Literal::Float64(bits), ExpressionType::Float64)?;
                Ok(Some(literal))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn binary(
        &mut self,
        id: NodeId,
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
    ) -> ExpressionResult<u32> {
        match op {
            BinaryOp::And | BinaryOp::Or => {
                let left = self.check(left, Some(&ExpressionType::Bool))?;
                let right = self.check(right, Some(&ExpressionType::Bool))?;
                self.emit(id, Op::Binary(op), ExpressionType::Bool, vec![left, right])
            }
            BinaryOp::Coalesce => self.coalesce(id, left, right),
            BinaryOp::Equal | BinaryOp::NotEqual => {
                let (left, right) = self.same_type(left, right)?;
                if let ExpressionType::Logic4(_) = self.ty(left) {
                    // Four-valued buses have two equalities; neither is implied.
                    return Err(self.deny(
                        id,
                        ExpressionDenialDetail::FunctionContractMismatch {
                            function: "==".to_string(),
                            reason: "compare Logic4 buses with case_equal or logic_eq",
                        },
                    ));
                }
                self.emit(id, Op::Binary(op), ExpressionType::Bool, vec![left, right])
            }
            BinaryOp::Less | BinaryOp::LessEqual | BinaryOp::Greater | BinaryOp::GreaterEqual => {
                let (left, right) = self.same_type(left, right)?;
                if !self.ty(left).is_ordered() {
                    return Err(self.expected_kind(id, "an ordered type", left));
                }
                self.emit(id, Op::Binary(op), ExpressionType::Bool, vec![left, right])
            }
            _ => self.arithmetic(id, op, left, right),
        }
    }

    fn arithmetic(
        &mut self,
        id: NodeId,
        op: BinaryOp,
        left: NodeId,
        right: NodeId,
    ) -> ExpressionResult<u32> {
        let left = self.check(left, None)?;
        let right = self.check(right, None)?;
        let ty = self.arithmetic_type(op, self.ty(left), self.ty(right));
        match ty {
            Ok(ty) => self.emit(id, Op::Binary(op), ty, vec![left, right]),
            Err(detail) => Err(self.deny(id, detail)),
        }
    }

    fn arithmetic_type(
        &self,
        op: BinaryOp,
        left: &ExpressionType,
        right: &ExpressionType,
    ) -> Result<ExpressionType, ExpressionDenialDetail> {
        use ExpressionType::{Decimal, Float64, Integer, Quantity};
        let bounds = || ExpressionDenialDetail::Bounds("dimension exponent out of range");
        let mismatch = || ExpressionDenialDetail::TypeMismatch {
            expected: left.to_string(),
            found: right.to_string(),
        };
        match (op, left, right) {
            (BinaryOp::Multiply, Quantity(a), Quantity(b)) => {
                a.multiply(*b).map(Quantity).ok_or_else(bounds)
            }
            (BinaryOp::Divide, Quantity(a), Quantity(b)) => {
                a.divide(*b).map(Quantity).ok_or_else(bounds)
            }
            (BinaryOp::Multiply | BinaryOp::Divide, Quantity(a), Float64) => Ok(Quantity(*a)),
            (BinaryOp::Multiply, Float64, Quantity(b)) => Ok(Quantity(*b)),
            _ if left != right => Err(mismatch()),
            (BinaryOp::Remainder, Integer(_), _) => Ok(left.clone()),
            (BinaryOp::Remainder, ..) => Err(ExpressionDenialDetail::TypeMismatch {
                expected: "an integer type".to_string(),
                found: left.to_string(),
            }),
            (BinaryOp::Divide, Decimal, _) => Err(ExpressionDenialDetail::UnsupportedFeature(
                "decimal division uses decimal_div with explicit rounding",
            )),
            (BinaryOp::Add | BinaryOp::Subtract, Quantity(_), _) => Ok(left.clone()),
            (BinaryOp::Multiply | BinaryOp::Divide, Quantity(_), _) => Err(mismatch()),
            _ if left.is_numeric() => Ok(left.clone()),
            _ => Err(ExpressionDenialDetail::TypeMismatch {
                expected: "a numeric type".to_string(),
                found: left.to_string(),
            }),
        }
    }

    fn coalesce(&mut self, id: NodeId, left: NodeId, right: NodeId) -> ExpressionResult<u32> {
        let left = self.check(left, None)?;
        let ExpressionType::Option(inner) = self.ty(left).clone() else {
            return Err(self.expected_kind(id, "an Option", left));
        };
        let right = if self.needs_expected(right) {
            self.check(right, Some(&inner))?
        } else {
            self.check(right, None)?
        };
        let ty = self.ty(right).clone();
        if ty != *inner && ty != ExpressionType::Option(inner.clone()) {
            return Err(self.mismatch(id, &inner, right));
        }
        self.emit(id, Op::Binary(BinaryOp::Coalesce), ty, vec![left, right])
    }

    /// Checks two operands that must share a type, inferring whichever side
    /// can be typed alone first.
    fn same_type(&mut self, left: NodeId, right: NodeId) -> ExpressionResult<(u32, u32)> {
        if self.needs_expected(left) && !self.needs_expected(right) {
            let right = self.check(right, None)?;
            let ty = self.ty(right).clone();
            let left = self.check(left, Some(&ty))?;
            return Ok((left, right));
        }
        let left = self.check(left, None)?;
        let ty = self.ty(left).clone();
        let right = self.check(right, Some(&ty))?;
        Ok((left, right))
    }

    pub(super) fn conditional(
        &mut self,
        id: NodeId,
        [condition, then, otherwise]: [NodeId; 3],
        expected: Option<&ExpressionType>,
    ) -> ExpressionResult<u32> {
        let condition = self.check(condition, Some(&ExpressionType::Bool))?;
        let (then, otherwise) = match expected {
            Some(expected) => (
                self.check(then, Some(expected))?,
                self.check(otherwise, Some(expected))?,
            ),
            None => self.same_type(then, otherwise)?,
        };
        let ty = self.ty(then).clone();
        self.emit(id, Op::Conditional, ty, vec![condition, then, otherwise])
    }

    pub(super) fn expected_kind(
        &self,
        id: NodeId,
        expected: &str,
        found: u32,
    ) -> crate::expressions::denial::ExpressionDenial {
        self.deny(
            id,
            ExpressionDenialDetail::TypeMismatch {
                expected: expected.to_string(),
                found: self.ty(found).to_string(),
            },
        )
    }
}
