use super::{
    attempt::begin, WorthQueryLoweredProvisionalEffectProgram, WorthQueryProposedFact,
    WorthQueryProvisionalAttempt, WorthQueryProvisionalDiscardOutcome,
    WorthQueryProvisionalFailure,
};

pub struct WorthQueryProposedPostState<'run> {
    pub(super) attempt: WorthQueryProvisionalAttempt<'run>,
}

impl<'run> WorthQueryProposedPostState<'run> {
    pub fn identity(&self) -> &str {
        self.attempt.overlay.evidence().proposed_state_identity()
    }

    pub fn generation(&self) -> u64 {
        self.attempt.generation
    }

    pub fn facts(&self) -> &[WorthQueryProposedFact] {
        self.attempt.overlay.evidence().facts()
    }

    pub fn inspect(self) -> WorthQueryProposedStateInspection<'run> {
        WorthQueryProposedStateInspection { proposed: self }
    }

    pub fn discard(self) -> WorthQueryProvisionalDiscardOutcome {
        self.attempt.discard()
    }
}

pub struct WorthQueryProposedStateInspection<'run> {
    pub(super) proposed: WorthQueryProposedPostState<'run>,
}

impl<'run> WorthQueryProposedStateInspection<'run> {
    pub fn proposed_state_identity(&self) -> &str {
        self.proposed.identity()
    }

    pub fn generation(&self) -> u64 {
        self.proposed.generation()
    }

    pub fn facts(&self) -> &[WorthQueryProposedFact] {
        self.proposed.facts()
    }

    pub fn revise(
        mut self,
        program: WorthQueryLoweredProvisionalEffectProgram,
    ) -> Result<WorthQueryProvisionalAttempt<'run>, WorthQueryProvisionalFailure> {
        self.proposed.attempt.counters.provider_discard_calls += 1;
        if let Err(failure) = self.proposed.attempt.overlay.discard() {
            let posture = self.proposed.attempt.staged.abort().recovery_posture();
            return Err(failure.with_recovery_posture(if posture
                == crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture::Closed
            {
                crate::domain_computation::provider_session::WorthQueryProviderSessionRecoveryPosture::RecoveryRequired
            } else {
                posture
            }));
        }
        begin(
            self.proposed.attempt.staged,
            self.proposed.attempt.read_set,
            program,
            self.proposed.attempt.generation + 1,
            self.proposed.attempt.counters,
        )
    }

    pub fn discard(self) -> WorthQueryProvisionalDiscardOutcome {
        self.proposed.discard()
    }
}
