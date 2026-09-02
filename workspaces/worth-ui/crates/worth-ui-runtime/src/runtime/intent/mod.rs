mod admission;
mod application_generation_identity;
mod attempt_lineage;
mod causal_trace;
mod confirmation;
mod operability;
mod payload;
mod routing;

pub use crate::declaration::UiIntentRouteResolutionCost;
pub(crate) use admission::{
    prepare_typed_admission_candidate, revalidate_typed_candidate_for_execution,
    validate_typed_inoperable, UiAdmittedIntentIdentity, UiCurrentIntentAdmissionCandidate,
    UiIntentAdmissionCurrentnessContext, UiIntentAdmissionLease, UiIntentAdmissionState,
    UiPreparedIntentAdmissionCandidate,
};
pub use admission::{
    UiAdmittedIntent, UiIntentAdmissionCancellationReason, UiIntentAdmissionCost,
    UiIntentAdmissionDecision, UiIntentAdmissionMetrics, UiIntentAdmissionSettlementPosture,
    UiIntentAdmissionSettlementReceipt, UiIntentAdmissionShutdownReport,
    UiIntentAdmissionSlotIdentity, UiIntentAdmissionStop, UiIntentAdmissionStopReason,
    UI_INTENT_ADMISSION_CAPACITY,
};
pub use application_generation_identity::WorthUiActiveApplicationGenerationIdentity;
pub use attempt_lineage::UiIntentAttemptLineage;
pub(crate) use causal_trace::UiIntentCausalTraceAdmissionPrefix;
pub(crate) use confirmation::{
    continue_confirmation, UiIntentConfirmationContinuationContext, UiIntentConfirmationState,
};
pub use confirmation::{
    UiConfirmedIntentCandidate, UiIntentConfirmationCancellationReason,
    UiIntentConfirmationChallenge, UiIntentConfirmationContinuation,
    UiIntentConfirmationIssueOutcome, UiIntentConfirmationLookupCost, UiIntentConfirmationMetrics,
    UiIntentConfirmationSettlementReceipt, UiIntentConfirmationShutdownReport,
    UiIntentConfirmationSlotIdentity, UiIntentConfirmationStop, UiIntentConfirmationStopReason,
    UiIntentConfirmationTimeBasisKind, UiPendingIntentConfirmation,
    UI_INTENT_CONFIRMATION_TTL_MILLIS, UI_PENDING_INTENT_CONFIRMATION_LIMIT,
};
#[cfg(any(test, feature = "certification-support"))]
pub(crate) use operability::UiIntentOperabilityDecisionInput;
pub(crate) use operability::{
    evaluate_intent_operability, UiIntentOccupancyPlacement, UiIntentOccupancyState,
};
pub use operability::{
    UiInoperableIntentCandidate, UiIntentAffinityPosture, UiIntentConfirmationPosture,
    UiIntentInoperableCause, UiIntentInoperableCauseIter, UiIntentMutabilityPosture,
    UiIntentOccupancyPosture, UiIntentOperabilityCost, UiIntentOperabilityDecision,
    UiIntentOperabilityOutcome, UiIntentOperabilityProof, UiIntentPolicyPosture,
    UiIntentReadinessPosture, UiIntentSupportPosture,
};
#[cfg(any(test, feature = "certification-support"))]
pub use operability::{
    UiIntentOccupancyReleasePosture, UiIntentOccupancyReservation,
    UiIntentOccupancyReservationDenial,
};
#[cfg(not(any(test, feature = "certification-support")))]
pub(crate) use operability::{UiIntentOccupancyReservation, UiIntentOccupancyReservationDenial};
pub(crate) use operability::{
    UiIntentOperabilityAppearanceClass, UiIntentOperabilityStandingFactSnapshot,
};
pub(crate) use payload::UiValidationAppearanceClass;
pub(crate) use payload::{
    prepare_intent_payload, UiIntentApplicationFactState, UiValidationAppearanceFactSnapshot,
};
#[cfg(test)]
pub(crate) use payload::{UiAdmittedValidationAppearanceTarget, UiValidationAppearanceFactDenial};
pub use payload::{
    UiIntentApplicationFactRevision, UiIntentApplicationFactUpdateDenial,
    UiIntentApplicationFactUpdateReceipt, UiIntentDraftInputRevision, UiIntentInputBasisReceipt,
    UiIntentInputOwnerRevision, UiIntentPayloadProjectionCost, UiIntentPayloadStop,
    UiIntentQueryInputRevision, UiPreparedIntentPayload,
};
pub(crate) use routing::resolve_intent_route;
pub use routing::{
    UiIntentRouteResolution, UiIntentRouteResolutionStop, UiResolvedConfirmationIntentRoute,
    UiResolvedProductIntentRoute,
};
