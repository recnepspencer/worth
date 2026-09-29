use worth_foundational::expression_api::{ExpressionDenial, ExpressionValue};
use worth_ui_dsl::{WorthUiExpressionResultType, WorthUiExpressionRole};
use worth_ui_query_binding::{
    UiProjectionFactStopKind, UiProjectionInputTransitionStopKind,
    UiProjectionRetainedActivityKind, UiProjectionUnavailableKind,
};

/// What one expression is right now. Denial, unavailability and staleness are
/// their own outcomes: none of them is ever `Condition(false)`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionOutcome {
    Condition(bool),
    Value(ExpressionValue),
    Denied(UiExpressionDenialReason),
    Unavailable(UiExpressionUnavailableReason),
    Stale {
        reason: UiExpressionStaleReason,
        /// The last outcome that was current before this expression went
        /// stale, kept as evidence and never as a current result.
        last_current: Option<UiExpressionCurrentValue>,
    },
}

/// A value an expression produced while it was current.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionCurrentValue {
    Condition(bool),
    Value(ExpressionValue),
}

/// Why an expression consumer holds no current result. Each keeps its own
/// posture: none of them is ever read as `false` or as a default value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiExpressionWithholding {
    /// The expression was denied, or produced something other than the type
    /// its consumer reads.
    Denied,
    /// An operand the expression reads has no current fact.
    Unavailable,
    /// The expression's result is retained from an operand that is no longer
    /// current, or belongs to a generation the reader does not observe.
    Stale,
}

/// The operand fact that stopped an expression, kept distinct per owner.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiExpressionStopKind {
    Projection(UiProjectionFactStopKind),
    Transition(UiProjectionInputTransitionStopKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionDenialReason {
    OperandWrongWorld {
        operand: Box<str>,
    },
    OperandStopped {
        operand: Box<str>,
        kind: UiExpressionStopKind,
    },
    /// A current scalar projection carried no value.
    OperandValueMissing {
        operand: Box<str>,
    },
    /// The observed operand is not the shape or kind the catalog installed.
    OperandShapeMismatch {
        operand: Box<str>,
    },
    /// The observed projection is not the one the catalog installed.
    OperandProjectionMismatch {
        operand: Box<str>,
    },
    Upstream {
        operand: Box<str>,
        identity: Box<str>,
    },
    Evaluation(ExpressionDenial),
    /// The kernel returned a value the expression's role cannot carry.
    RoleMismatch {
        role: WorthUiExpressionRole,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionUnavailableReason {
    Projection {
        operand: Box<str>,
        kind: UiProjectionUnavailableKind,
    },
    ProjectionAbsent {
        operand: Box<str>,
    },
    /// The application fact owner of this generation holds no value for the
    /// fact the operand names.
    ApplicationFactAbsent {
        operand: Box<str>,
        fact: Box<str>,
    },
    Upstream {
        operand: Box<str>,
        identity: Box<str>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiExpressionStaleReason {
    ProjectionRetained {
        operand: Box<str>,
        kind: UiProjectionRetainedActivityKind,
    },
    Upstream {
        operand: Box<str>,
        identity: Box<str>,
    },
}

impl UiExpressionOutcome {
    /// The outcome a kernel value is for an expression of `role`. A value
    /// whose kernel type is not the role's is a role-mismatch denial, never
    /// coerced: a condition yields only a truth value, and a derived value
    /// only its declared result type.
    pub(super) fn of_role(role: WorthUiExpressionRole, value: &ExpressionValue) -> Self {
        match role {
            WorthUiExpressionRole::Condition => match value.as_bool() {
                Some(condition) => Self::Condition(condition),
                None => Self::Denied(UiExpressionDenialReason::RoleMismatch { role }),
            },
            WorthUiExpressionRole::Derived(result) if carries(result, value) => {
                Self::Value(value.clone())
            }
            WorthUiExpressionRole::Derived(_) => {
                Self::Denied(UiExpressionDenialReason::RoleMismatch { role })
            }
        }
    }

    /// The value a current outcome carries. Every other outcome carries none.
    pub fn current_value(&self) -> Option<UiExpressionCurrentValue> {
        match self {
            Self::Condition(value) => Some(UiExpressionCurrentValue::Condition(*value)),
            Self::Value(value) => Some(UiExpressionCurrentValue::Value(value.clone())),
            Self::Denied(_) | Self::Unavailable(_) | Self::Stale { .. } => None,
        }
    }

    /// The truth value a condition consumer may act on. A condition slot
    /// never records `Value`: the evaluator records a non-boolean kernel
    /// result as a role-mismatch denial, so a `Value` read here is withheld as
    /// denied rather than coerced.
    pub const fn condition(&self) -> Result<bool, UiExpressionWithholding> {
        match self {
            Self::Condition(value) => Ok(*value),
            Self::Value(_) | Self::Denied(_) => Err(UiExpressionWithholding::Denied),
            Self::Unavailable(_) => Err(UiExpressionWithholding::Unavailable),
            Self::Stale { .. } => Err(UiExpressionWithholding::Stale),
        }
    }

    /// The text a derived consumer may act on. The evaluator records a
    /// derived value of any other kernel type as a role-mismatch denial, so a
    /// non-text value read here is withheld as denied rather than coerced.
    pub fn derived_text(&self) -> Result<&str, UiExpressionWithholding> {
        match self {
            Self::Value(value) => value.as_str().ok_or(UiExpressionWithholding::Denied),
            Self::Condition(_) | Self::Denied(_) => Err(UiExpressionWithholding::Denied),
            Self::Unavailable(_) => Err(UiExpressionWithholding::Unavailable),
            Self::Stale { .. } => Err(UiExpressionWithholding::Stale),
        }
    }

    /// The integer a derived consumer may act on, as the kernel carries it.
    /// Whether it fits the consumer's own range is the consumer's check.
    pub fn derived_integer(&self) -> Result<i128, UiExpressionWithholding> {
        match self {
            Self::Value(value) => value.as_integer().ok_or(UiExpressionWithholding::Denied),
            Self::Condition(_) | Self::Denied(_) => Err(UiExpressionWithholding::Denied),
            Self::Unavailable(_) => Err(UiExpressionWithholding::Unavailable),
            Self::Stale { .. } => Err(UiExpressionWithholding::Stale),
        }
    }

    pub const fn is_current(&self) -> bool {
        match self {
            Self::Condition(_) | Self::Value(_) => true,
            Self::Denied(_) | Self::Unavailable(_) | Self::Stale { .. } => false,
        }
    }
}

/// Whether `value` has the kernel type a derived `result` declares.
fn carries(result: WorthUiExpressionResultType, value: &ExpressionValue) -> bool {
    match result {
        WorthUiExpressionResultType::Text | WorthUiExpressionResultType::Token => {
            value.as_str().is_some()
        }
        WorthUiExpressionResultType::Integer => value
            .as_integer()
            .is_some_and(|integer| i64::try_from(integer).is_ok()),
        WorthUiExpressionResultType::Decimal => value.as_decimal().is_some(),
    }
}

#[cfg(test)]
mod tests;
