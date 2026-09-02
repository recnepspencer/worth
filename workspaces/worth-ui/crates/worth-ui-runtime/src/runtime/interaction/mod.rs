mod batch;
pub(crate) mod draft;
pub(crate) mod gesture;
mod pointer_presence;
mod semantic;
mod service_event;
mod settlement;
mod snapshot;
mod source;
mod state;
pub(crate) mod targeting;
mod transition;

pub use batch::{
    UiHostInteractionIngressOutcome, UiInteractionBatchReceipt, UiInteractionObservationDenial,
    UiInteractionShutdownReport, UiQuarantinedHostInteractionBatch,
};
pub use draft::{
    UiDraftByteBudget, UiDraftByteBudgetDenial, UiDraftFieldIdentity, UiDraftMutationKind,
    UiDraftMutationReceipt, UiDraftRecipientContractDenial, UiDraftSessionIdentity,
    UiLocalInputRecipientAdmission, UiLocalInputRecipientBindingReceipt,
    UiLocalInputRecipientBindingStop, UiLocalInputRecipientBindingStopReason,
    UiLocalInputRecipientContract, UiLocalInputRecipientFamily, UiLocalInputStop,
    UiLocalInputStopReason, UI_DRAFT_SESSION_LIMIT, UI_DRAFT_UTF8_BYTE_LIMIT,
};
pub use gesture::{
    UiPointerGestureContinuityKind, UiPointerGesturePressReceipt, UiPointerGestureStop,
    UiPointerGestureStopReason, UiTargetedPointerGesture, UI_ACTIVE_POINTER_GESTURE_LIMIT,
};
pub use pointer_presence::UiPointerPresenceTargetTransition;
pub use pointer_presence::UiPointerPresenceAdmissionDenial;
pub(crate) use pointer_presence::{
    UiPointerPresenceAppearanceOwnerSnapshot, UiPointerPresenceAppearancePosture,
    UiPointerPresenceCapacity, UiPointerPresenceClass, UiPointerPresencePresentationTrigger,
    UiPointerPresencePresentationTriggerDenial, UiPointerPresenceOwner, UiPrimaryPointerKind,
};
pub(crate) use semantic::{
    selection_evidence_input, semantic_evidence_input, UiEditCommitInput, UiKeyboardSemanticInput,
};
pub use semantic::{
    UiActivateInteraction, UiActivateInteractionSource, UiEditCommitInteraction,
    UiKeyboardActivationEvidence, UiSelectionCommitInteraction, UiSelectionCommitStop,
    UiSelectionCommitStopReason, UiSemanticInteraction, UiSubmitInteraction,
};
pub use service_event::{UiDismissInteraction, UiDismissInteractionCause};
pub use settlement::UiInteractionLifecycleSettlementReceipt;
pub use snapshot::{UiInteractionLifecycleCounters, UiInteractionStateSnapshot};
pub use source::UiIntentRouteSource;
pub(crate) use source::{command_evidence_input, UiIntentRouteSourceMaterial};
pub(crate) use state::{UiInteractionLifecycleStopReason, UiInteractionRuntimeState};
pub use targeting::{
    UiInteractionTargetingDenial, UiPresentedInteractionTarget, UiPresentedInteractionTargetView,
    UiPresentedTargetFrameRelation,
};
pub(crate) use targeting::{UiPresentedInteractionGeometry, UiPresentedViewportGeometry};
pub use transition::{UiInteractionStop, UiInteractionTransition};
