//! Typed refusals raised while a where filter is admitted, before execution.
use crate::application_schema::ApplicationFieldBindingLocus;
use worth_foundational::expression_api::ExpressionDenial;

/// Why a where filter's operands or expression were refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationQueryWhereFilterDenial {
    /// An operand name is empty or carries surrounding whitespace.
    InvalidOperandName,
    DuplicateOperand,
    /// Two comparison clauses compare the exact same field, operator, and right-hand operand.
    DuplicateClause,
    /// A filter reads at least one row field or parameter.
    NoOperands,
    /// Every declared operand must be read by the expression.
    UnusedOperand,
    /// The field selector does not resolve to a declared record field.
    InvalidRecordField {
        field: ApplicationFieldBindingLocus,
    },
    /// Unit-bearing values need a quantity contract; they never silently
    /// become unitless scalars.
    UnitBearingOperand,
    /// The scalar family has no lossless expression correspondence.
    UnsupportedScalar,
    /// A literal operand value does not match its scalar family, cannot be
    /// converted losslessly, or exceeds the filter profile bound.
    InvalidLiteralValue,
    /// Foundation refused the draft: unknown names, type mismatches, a
    /// non-Bool result or a resource ceiling.
    Expression(ExpressionDenial),
}

impl From<ExpressionDenial> for ApplicationQueryWhereFilterDenial {
    fn from(denial: ExpressionDenial) -> Self {
        Self::Expression(denial)
    }
}

impl std::fmt::Display for ApplicationQueryWhereFilterDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidOperandName => formatter.write_str("where filter operand name is invalid"),
            Self::DuplicateOperand => formatter.write_str("where filter operand is duplicated"),
            Self::DuplicateClause => formatter.write_str("where filter clause is duplicated"),
            Self::NoOperands => formatter.write_str("where filter reads no field or parameter"),
            Self::UnusedOperand => formatter.write_str("where filter operand is never read"),
            Self::InvalidRecordField { field } => write!(
                formatter,
                "where filter field {}.{}.{} is not a declared record field",
                field.entity(),
                field.aspect(),
                field.field()
            ),
            Self::UnitBearingOperand => {
                formatter.write_str("where filter operand carries a physical unit")
            }
            Self::UnsupportedScalar => {
                formatter.write_str("where filter operand scalar family is unsupported")
            }
            Self::InvalidLiteralValue => {
                formatter.write_str("where filter literal operand value is invalid")
            }
            Self::Expression(denial) => {
                write!(formatter, "where filter expression denied: {denial:?}")
            }
        }
    }
}

impl std::error::Error for ApplicationQueryWhereFilterDenial {}
