use worth_signal::facade::branch::AdmittedSignalBranchBasis;

/// Signal owner posture for one publication. Mutation input is borrowed later
/// at execution time; this plan does not retain a callback or caller context.
/// Branch creation is not a publication posture and has its own plan
/// vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalComponentPlanPosture {
    RetainExact,
    AdvanceExact,
    /// Adopt the exact advanced owner result retained by a recovery record.
    /// This posture performs no further Signal owner mutation.
    AdoptSettled,
}

#[derive(Debug)]
pub struct SignalComponentPlan {
    posture: SignalComponentPlanPosture,
    expected: AdmittedSignalBranchBasis,
    adoption: Option<crate::publication::SignalAttemptProgress>,
}

impl SignalComponentPlan {
    pub(crate) fn adopt_settled(
        expected: AdmittedSignalBranchBasis,
        progress: crate::publication::SignalAttemptProgress,
    ) -> Self {
        Self {
            posture: SignalComponentPlanPosture::AdoptSettled,
            expected,
            adoption: Some(progress),
        }
    }

    pub(crate) fn settled_adoption(&self) -> Option<&crate::publication::SignalAttemptProgress> {
        self.adoption.as_ref()
    }
    pub const fn posture(&self) -> SignalComponentPlanPosture {
        self.posture
    }

    pub const fn expected(&self) -> &AdmittedSignalBranchBasis {
        &self.expected
    }

    pub(crate) fn retain_exact(expected: AdmittedSignalBranchBasis) -> Self {
        Self {
            posture: SignalComponentPlanPosture::RetainExact,
            expected,
            adoption: None,
        }
    }

    pub(crate) fn advance_exact(expected: AdmittedSignalBranchBasis) -> Self {
        Self {
            posture: SignalComponentPlanPosture::AdvanceExact,
            expected,
            adoption: None,
        }
    }
}
