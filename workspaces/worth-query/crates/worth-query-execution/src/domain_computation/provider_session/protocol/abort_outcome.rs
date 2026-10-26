//! Terminal custody of a staged provider abort.
use super::{
    WorthQueryClosedProviderSessionDisposition, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionRecoveryPosture,
};

#[derive(Debug)]
pub enum WorthQuerySessionAbortOutcome {
    Aborted(WorthQueryClosedProviderSessionDisposition),
    AbortRecoveryRequired(WorthQueryProviderSessionFailure),
}

impl WorthQuerySessionAbortOutcome {
    pub fn recovery_posture(&self) -> WorthQueryProviderSessionRecoveryPosture {
        match self {
            Self::Aborted(_) => WorthQueryProviderSessionRecoveryPosture::Closed,
            Self::AbortRecoveryRequired(_) => {
                WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
            }
        }
    }

    pub fn failure(&self) -> Option<&WorthQueryProviderSessionFailure> {
        match self {
            Self::Aborted(_) => None,
            Self::AbortRecoveryRequired(failure) => Some(failure),
        }
    }
}
