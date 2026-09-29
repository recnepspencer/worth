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

    pub const fn is_current(&self) -> bool {
        match self {
            Self::Condition(_) | Self::Value(_) => true,
            Self::Denied(_) | Self::Unavailable(_) | Self::Stale { .. } => false,
        }
    }
}
