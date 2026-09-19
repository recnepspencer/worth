//! Runtime lifecycle lanes: launch → replacement → planning → activation → execution → host observation.

pub(crate) mod activation;
pub(crate) use activation::committed_allocation_attempt::UiCommittedAllocationActivationAttempt;
pub(crate) use activation::UiAllocationCatalogDeltaActivationInput;
pub(crate) use activation::WorthUiApplicationPlanSwap;
pub(crate) use activation::WorthUiInitialMountedAllocationActivationDenial;
pub(crate) use activation::WorthUiPreparedApplicationPlanSwap;
pub(crate) use activation::WorthUiPreparedApplicationPublication;
pub(crate) use activation::WorthUiPreparedQueryAwarePlanOutcome;
mod active;
pub(crate) use active::WorthUiActiveExecutionPlan;
mod allocation_catalog_successor;
mod allocation_frame_dispatch;
pub use allocation_catalog_successor::{
    UiAllocationCatalogDeltaClosureDenial, UiAllocationCatalogSuccessorReceipt,
};
mod allocation_receipt;
pub(crate) use allocation_receipt::{
    UiCommittedOverlayExtentBounds, UiMountedOverlayExtentOwner, UiMountedOverlayRegionExtent,
};
mod application_item;
pub(crate) use application_item::{UiApplicationItemKey, UiApplicationItemKeyFamily};
pub(crate) mod appearance;
mod drag_resize;
pub(crate) mod execution;
pub(crate) mod exports;
pub(crate) mod host_observation;
pub(crate) mod intent;
pub(crate) mod pointer_affordance;
pub use intent::WorthUiActiveApplicationGenerationIdentity;
pub(crate) mod intent_execution;
pub(crate) mod interaction;
mod invalidation_narrowing;
pub(crate) use invalidation_narrowing::{
    UiAdmittedScrollInvalidationBinding, UiAllocationInvalidationAuthority,
};
mod launch;
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use launch::WorthUiActivationStagingPlans;
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use launch::WorthUiRuntimeLaunchAuthority;
pub(crate) use launch::{
    WorthUiInitialMountedCatalogPreparationDenial, WorthUiMountedAllocationActivationBasis,
};
pub(crate) mod command_routing;
pub use command_routing::{
    UiCommandAmbiguity, UiCommandInvocationOrigin, UiCommandPrefixReceipt, UiCommandRouteLoss,
    UiCommandRouteLossReason, UiCommandRouteReceipt, UiCommandRoutingOutcome,
    UiCommandRoutingSuppression,
};
pub(crate) mod focus;
mod measurement;
pub(crate) mod motion;
pub(crate) mod observation;
pub(crate) mod overlay_composition;
pub(crate) mod persistent_index;
pub(crate) mod planning;
pub(crate) mod portal;
pub(crate) mod presentation_state;
pub(crate) mod rebind;
pub mod replacement;
pub(crate) mod scroll;
pub(crate) mod selection;
mod service_installation;
mod service_state_persistence;
pub(crate) use service_installation::UiRuntimeServiceInstallation;
pub(crate) use service_state_persistence::UiServiceStatePersistencePosture;
pub(crate) mod session;
mod source_ingress;
pub(crate) use source_ingress::WorthUiAuthoredSourceBasis;
pub use source_ingress::{
    UiSourceCompilationDenialReceipt, UiSourceRebindAttemptBasis, UiSourceRebindAttemptDenial,
    UiSourceRebindAttemptDenialReceipt, UiSourceRebindAttemptFailure, UiSourceRebindAttemptOutcome,
};
mod stream_policy;
mod viewport_resize;

pub use drag_resize::*;
pub use scroll::allocation::{
    UiActivatedScrollOwner, UiActivatedScrollProjectionTarget, UiScrollContractAdmissionDenial,
    UiScrollOffsetAllocationPosture, UiScrollReceiptActivationKey, UiScrollVirtualizationPosture,
};
pub(crate) use scroll::allocation::{
    UiAdmittedScrollExtentSource, UiAdmittedScrollOwnedContract, UiScrollProjectionOwnerIdentity,
};

pub(crate) use allocation_frame_dispatch::UiPendingMountedPreviewTransition;
pub(crate) use allocation_receipt::project_allocation_preview;
pub(crate) use allocation_receipt::UiAllocationTruthRevision;
pub(crate) use allocation_receipt::UiCommittedAllocationCatalogActivationRow;
pub(crate) use allocation_receipt::UiCommittedScrollActivationSource;
pub(crate) use allocation_receipt::UiCommittedViewportGeometry;
pub(crate) use allocation_receipt::UiMountedAllocationExactDelta;
pub(crate) use allocation_receipt::UiMountedAllocationProjectionCatalog;
pub(crate) use allocation_receipt::UiMountedAllocationProjectionDelta;
pub(crate) use allocation_receipt::UiMountedAllocationProjectionDenial;
pub(crate) use allocation_receipt::UiMountedAllocationProjectionSource;
pub use exports::*;

#[cfg(test)]
pub(crate) use replacement::state_inventory::WorthUiTransientInteractionAdmission;

#[cfg(test)]
pub(crate) mod tests;
