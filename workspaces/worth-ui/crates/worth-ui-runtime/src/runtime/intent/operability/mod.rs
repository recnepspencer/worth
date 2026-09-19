mod axes;
mod basis;
mod decision;
mod evaluation;
mod occupancy;
mod proof;
#[allow(
    dead_code,
    reason = "milestone 3.16 Gate 0 seals operability appearance standing facts before consumption"
)]
mod standing_fact;
mod standing_observation;

pub use axes::{
    UiIntentAffinityPosture, UiIntentConfirmationPosture, UiIntentMutabilityPosture,
    UiIntentOccupancyPosture, UiIntentPolicyPosture, UiIntentReadinessPosture,
    UiIntentSupportPosture,
};
pub(crate) use basis::{
    observe_operability_basis, UiIntentOperabilityBasis, UiIntentOperabilityDependencyDrift,
};
pub(crate) use decision::UiIntentOperabilityDecisionInput;
pub use decision::{
    UiIntentInoperableCause, UiIntentInoperableCauseIter, UiIntentOperabilityCost,
    UiIntentOperabilityDecision,
};
pub(crate) use evaluation::evaluate_intent_operability;
#[cfg(any(test, feature = "certification-support"))]
pub use occupancy::UiIntentOccupancyReleasePosture;
pub(crate) use occupancy::{
    UiIntentOccupancyObservation, UiIntentOccupancyPlacement, UiIntentOccupancyState,
};
#[cfg(any(test, feature = "certification-support"))]
pub use occupancy::{UiIntentOccupancyReservation, UiIntentOccupancyReservationDenial};
#[cfg(not(any(test, feature = "certification-support")))]
pub(crate) use occupancy::{UiIntentOccupancyReservation, UiIntentOccupancyReservationDenial};
pub use proof::{
    UiInoperableIntentCandidate, UiIntentOperabilityOutcome, UiIntentOperabilityProof,
};
#[allow(
    unused_imports,
    reason = "milestone 3.16 Gate 0 exposes sealed operability appearance facts internally"
)]
pub(crate) use standing_fact::{
    UiIntentOperabilityAppearanceClass, UiIntentOperabilityStandingFact,
};
pub(crate) use standing_observation::{
    observe_activation_operability, UiIntentStandingOperabilityObservation,
    UiIntentStandingOperabilityUnavailable,
};
