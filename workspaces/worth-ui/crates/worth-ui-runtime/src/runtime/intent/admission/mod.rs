mod admitted;
mod candidate;
mod currentness;
mod decision;
mod metrics;
mod settlement;
mod standing_owner;
mod state;
mod stop;

pub(crate) use admitted::UiAdmittedIntentIdentity;
pub use admitted::{UiAdmittedIntent, UiIntentAdmissionSlotIdentity};
pub(crate) use candidate::{UiCurrentIntentAdmissionCandidate, UiPreparedIntentAdmissionCandidate};
pub(crate) use currentness::{
    prepare_typed_candidate as prepare_typed_admission_candidate,
    revalidate_typed_candidate_for_execution, validate_typed_inoperable,
    UiIntentAdmissionCurrentnessContext,
};
pub use decision::UiIntentAdmissionDecision;
pub use metrics::{UiIntentAdmissionMetrics, UiIntentAdmissionShutdownReport};
pub(crate) use settlement::UiIntentAdmissionLease;
pub use settlement::{
    UiIntentAdmissionCancellationReason, UiIntentAdmissionSettlementPosture,
    UiIntentAdmissionSettlementReceipt,
};
pub(crate) use standing_owner::UiIntentOperabilityStandingFactSnapshot;
pub(crate) use state::UiIntentAdmissionState;
pub use stop::{UiIntentAdmissionCost, UiIntentAdmissionStop, UiIntentAdmissionStopReason};

pub const UI_INTENT_ADMISSION_CAPACITY: usize = 16;
