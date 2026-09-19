mod application_scalar;
mod budget;
mod collection;
mod collection_change_derivation;
mod collection_derivation;
mod collection_work_counters;
mod fact_receipt;
mod intent_input;
mod native_value;
mod posture;
mod scalar;
mod scalar_derivation;
mod work_counters;

pub use application_scalar::UiApplicationScalarProjectionFactReceipt;
pub use budget::{
    UiCollectionProjectionBudget, UiCollectionProjectionBudgetError, UiProjectionConsumptionBudget,
    UiProjectionConsumptionBudgetError, UiProjectionConsumptionLimits,
};
pub use collection::{
    UiCollectionCompleteness, UiCollectionContinuation, UiCollectionProjectionChange,
    UiCollectionProjectionDelivery, UiCollectionProjectionFactReceipt,
    UiCollectionProjectionRowReference, UiCollectionProjectionTextRow, UiCollectionProjectionValue,
};
pub(crate) use collection_change_derivation::derive_applied_collection_projection;
pub(crate) use collection_derivation::{
    derive_collection_projection, derive_initial_collection_projection,
    UiCollectionDerivationContext,
};
pub use collection_work_counters::UiCollectionProjectionWorkCounters;
pub use fact_receipt::UiProjectionFactReceipt;
pub(crate) use fact_receipt::UiProjectionFactReceiptInput;
pub use intent_input::{
    UiCollectionProjectionInputFact, UiProjectionInputCollectionRow,
    UiProjectionInputFactReference, UiProjectionInputFactTransition, UiProjectionInputPosture,
    UiProjectionInputRevision, UiProjectionInputSlot, UiProjectionInputTransitionStopKind,
    UiProjectionInputTransitionWork, UiProjectionOptionReference, UiProjectionOptionStableKey,
    UiScalarProjectionInputFact,
};
pub use native_value::UiNativeTextValue;
pub use posture::{
    UiPresentProjection, UiProjectionAvailability, UiProjectionFactStopKind,
    UiProjectionFactStopReceipt, UiProjectionPostureTrace, UiProjectionRetainedActivityKind,
    UiProjectionRetainedActivityReceipt, UiProjectionTransitionPosture,
    UiProjectionUnavailableKind, UiProjectionUnavailableReceipt,
};
pub use scalar::UiScalarProjectionFactReceipt;
pub(crate) use scalar_derivation::derive_scalar_projection;
pub use work_counters::UiScalarProjectionWorkCounters;
