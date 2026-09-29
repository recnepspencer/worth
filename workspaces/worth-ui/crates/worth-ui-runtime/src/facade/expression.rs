//! Stable product-facing expression evaluation records and counters.

pub use crate::runtime::expression::{
    UiExpressionCatalogPreparationDenial, UiExpressionConditionWithholding,
    UiExpressionCurrentValue, UiExpressionDenialReason, UiExpressionEvaluationRecord,
    UiExpressionOperandFact, UiExpressionOutcome, UiExpressionResultReference, UiExpressionSlot,
    UiExpressionStaleReason, UiExpressionStopKind, UiExpressionUnavailableReason,
    UiExpressionWorkCounters,
};
pub use crate::runtime::intent::{
    UiIntentApplicationInputReference, UiIntentApplicationInputRevision,
};
