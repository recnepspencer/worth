use worth_foundational::expression_api::{ExpressionDenial, ExpressionValue};
use worth_ui_dsl::WorthUiExpressionRole;
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

/// Why a condition consumer holds no truth value. Each keeps its own posture:
/// none of them is ever read as `false`.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum UiExpressionConditionWithholding {
    /// The condition was denied, or produced something other than a truth value.
    Denied,
    /// An operand the condition reads has no current fact.
    Unavailable,
    /// The condition's result is retained from an operand that is no longer
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
    pub const fn condition(&self) -> Result<bool, UiExpressionConditionWithholding> {
        match self {
            Self::Condition(value) => Ok(*value),
            Self::Value(_) | Self::Denied(_) => Err(UiExpressionConditionWithholding::Denied),
            Self::Unavailable(_) => Err(UiExpressionConditionWithholding::Unavailable),
            Self::Stale { .. } => Err(UiExpressionConditionWithholding::Stale),
        }
    }

    pub const fn is_current(&self) -> bool {
        match self {
            Self::Condition(_) | Self::Value(_) => true,
            Self::Denied(_) | Self::Unavailable(_) | Self::Stale { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_truth_value_is_a_condition_and_every_other_outcome_keeps_its_withholding() {
        let operand: Box<str> = "f".into();
        for (outcome, expected) in [
            (UiExpressionOutcome::Condition(true), Ok(true)),
            (UiExpressionOutcome::Condition(false), Ok(false)),
            (
                UiExpressionOutcome::Value(ExpressionValue::bool(false)),
                Err(UiExpressionConditionWithholding::Denied),
            ),
            (
                UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandShapeMismatch {
                    operand: operand.clone(),
                }),
                Err(UiExpressionConditionWithholding::Denied),
            ),
            (
                UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
                    operand: operand.clone(),
                }),
                Err(UiExpressionConditionWithholding::Unavailable),
            ),
            (
                UiExpressionOutcome::Stale {
                    reason: UiExpressionStaleReason::ProjectionRetained {
                        operand: operand.clone(),
                        kind: UiProjectionRetainedActivityKind::Idle,
                    },
                    last_current: Some(UiExpressionCurrentValue::Condition(true)),
                },
                Err(UiExpressionConditionWithholding::Stale),
            ),
        ] {
            assert_eq!(outcome.condition(), expected, "{outcome:?}");
        }
    }
}
