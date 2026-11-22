//! Direct-lane iteration start denial and terminal transitions.

use super::super::WorthQueryConvergenceEpochDenial;
use super::{
    association::DirectAssociatedStartRejection, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryIteratingDirectConvergenceEpoch,
};

pub struct WorthQueryDirectConvergenceIterationStartRejection {
    rejection: DirectAssociatedStartRejection,
}

pub struct WorthQueryDirectConvergenceIterationStartTermination {
    denial: WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryDirectConvergenceIterationOutcome,
}

impl WorthQueryDirectConvergenceIterationStartRejection {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        self.rejection.denial()
    }

    pub fn into_epoch(self) -> WorthQueryIteratingDirectConvergenceEpoch {
        WorthQueryIteratingDirectConvergenceEpoch {
            association: self.rejection.into_epoch(),
        }
    }

    pub fn terminate(self) -> WorthQueryDirectConvergenceIterationStartTermination {
        self.rejection.terminate()
    }
}

impl WorthQueryDirectConvergenceIterationStartTermination {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub fn outcome(&self) -> &WorthQueryDirectConvergenceIterationOutcome {
        &self.outcome
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryConvergenceEpochDenial,
        WorthQueryDirectConvergenceIterationOutcome,
    ) {
        (self.denial, self.outcome)
    }
}

pub(super) fn admit_start_rejection(
    rejection: DirectAssociatedStartRejection,
) -> WorthQueryDirectConvergenceIterationStartRejection {
    WorthQueryDirectConvergenceIterationStartRejection { rejection }
}

pub(super) fn start_termination(
    denial: WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryDirectConvergenceIterationOutcome,
) -> WorthQueryDirectConvergenceIterationStartTermination {
    WorthQueryDirectConvergenceIterationStartTermination { denial, outcome }
}
