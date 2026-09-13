use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentAttemptObservationReference;

pub(super) struct Ready;

pub(super) struct FirstHeld {
    pub(super) attempt: PlatformPulseIntentAttemptObservationReference,
}

pub(super) struct FirstCompleted;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ConfirmationChallenge {
    pub(super) slot: u8,
    pub(super) generation: u64,
    pub(super) lineage: u64,
    pub(super) expires_at_millis: u64,
}

pub(super) struct ConfirmationPending {
    pub(super) challenge: ConfirmationChallenge,
    pub(super) control: crate::adjudication::PlatformPulseConfirmationControlPoint,
}

pub(super) struct ConfirmationStale {
    pub(super) predecessor: ConfirmationChallenge,
    pub(super) control: crate::adjudication::PlatformPulseConfirmationControlPoint,
}

pub(super) struct FreshConfirmationPending {
    pub(super) challenge: ConfirmationChallenge,
    pub(super) control: crate::adjudication::PlatformPulseConfirmationControlPoint,
}

pub(super) struct SecondCompleted;

pub(super) struct DisabledStopped;

pub(super) struct PolicyDeniedStopped;

pub(super) struct FinalHeld {
    pub(super) attempt: PlatformPulseIntentAttemptObservationReference,
}

pub(super) struct Cancelled;
