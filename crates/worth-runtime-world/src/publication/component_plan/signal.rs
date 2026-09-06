use worth_signal::facade::branch::AdmittedSignalBranchBasis;

/// Signal owner posture for one publication. Mutation input is borrowed later
/// at execution time; this plan does not retain a callback or caller context.
/// Branch creation is not a publication posture and has its own plan
/// vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalComponentPlanPosture {
    RetainExact,
    AdvanceExact,
}

#[derive(Debug)]
pub struct SignalComponentPlan {
    posture: SignalComponentPlanPosture,
    expected: AdmittedSignalBranchBasis,
}

impl SignalComponentPlan {
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
        }
    }

    pub(crate) fn advance_exact(expected: AdmittedSignalBranchBasis) -> Self {
        Self {
            posture: SignalComponentPlanPosture::AdvanceExact,
            expected,
        }
    }
}
