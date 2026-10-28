use super::WorthQueryProvisionalFailure;
use crate::domain_computation::provider_session::{
    WorthQueryProviderSessionRecoveryPosture, WorthQuerySessionAbortOutcome,
};

pub struct WorthQueryProvisionalDiscardOutcome {
    overlay_failure: Option<WorthQueryProvisionalFailure>,
    session_outcome: WorthQuerySessionAbortOutcome,
}

impl WorthQueryProvisionalDiscardOutcome {
    pub(crate) fn new(
        overlay_failure: Option<WorthQueryProvisionalFailure>,
        session_outcome: WorthQuerySessionAbortOutcome,
    ) -> Self {
        Self {
            overlay_failure,
            session_outcome,
        }
    }

    pub fn overlay_failure(&self) -> Option<&WorthQueryProvisionalFailure> {
        self.overlay_failure.as_ref()
    }

    pub fn recovery_posture(&self) -> WorthQueryProviderSessionRecoveryPosture {
        if self.overlay_failure.is_some() {
            WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
        } else {
            self.session_outcome.recovery_posture()
        }
    }

    pub fn session_outcome(&self) -> &WorthQuerySessionAbortOutcome {
        &self.session_outcome
    }
}
