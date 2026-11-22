//! Workflow-lane iteration start denial and terminal transitions.

use super::super::WorthQueryConvergenceEpochDenial;
use super::{
    association::WorkflowAssociatedIterationStartRejection,
    WorthQueryIteratingWorkflowConvergenceEpoch, WorthQueryWorkflowConvergenceIterationOutcome,
};

pub struct WorthQueryWorkflowConvergenceIterationStartRejection {
    rejection: WorkflowAssociatedIterationStartRejection,
}

pub struct WorthQueryWorkflowConvergenceIterationStartTermination {
    denial: WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryWorkflowConvergenceIterationOutcome,
}

impl WorthQueryWorkflowConvergenceIterationStartRejection {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        self.rejection.denial()
    }

    pub fn into_epoch(self) -> WorthQueryIteratingWorkflowConvergenceEpoch {
        WorthQueryIteratingWorkflowConvergenceEpoch {
            association: self.rejection.into_epoch(),
        }
    }

    pub fn terminate(self) -> WorthQueryWorkflowConvergenceIterationStartTermination {
        self.rejection.terminate()
    }
}

impl WorthQueryWorkflowConvergenceIterationStartTermination {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub fn outcome(&self) -> &WorthQueryWorkflowConvergenceIterationOutcome {
        &self.outcome
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryConvergenceEpochDenial,
        WorthQueryWorkflowConvergenceIterationOutcome,
    ) {
        (self.denial, self.outcome)
    }
}

pub(super) fn admit_start_rejection(
    rejection: WorkflowAssociatedIterationStartRejection,
) -> WorthQueryWorkflowConvergenceIterationStartRejection {
    WorthQueryWorkflowConvergenceIterationStartRejection { rejection }
}

pub(super) fn start_termination(
    denial: WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryWorkflowConvergenceIterationOutcome,
) -> WorthQueryWorkflowConvergenceIterationStartTermination {
    WorthQueryWorkflowConvergenceIterationStartTermination { denial, outcome }
}
