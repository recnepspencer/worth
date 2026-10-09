use super::super::WorthQueryConvergenceEpochCounters;
use super::association::{
    WorkflowAdmittedEpochAssociation, WorkflowAssociatedStepOutcome,
    WorkflowIteratingEpochAssociation, WorkflowIterationAssociation,
    WorkflowStartRejectionAssociation,
};
use super::completion::WorthQueryWorkflowConvergenceIterationOutcome;
use super::start::{
    WorthQueryWorkflowConvergenceIterationStartRejection,
    WorthQueryWorkflowConvergenceIterationStartTermination,
};
use super::yield_transition::WorthQueryWorkflowConvergenceYieldOutcome;
use super::{start, yield_transition};
use crate::domain_computation::{
    WorthQueryActiveWorkflowGraphExecution, WorthQueryManagedGraphCallRequest,
    WorthQueryPausedWorkflowGraphExecution, WorthQueryPendingWorkflowGraphChunk,
    WorthQueryWorkflowRunStartRejection,
};

pub struct WorthQueryAdmittedWorkflowConvergenceEpoch {
    pub(super) association: WorkflowAdmittedEpochAssociation,
}

impl WorthQueryAdmittedWorkflowConvergenceEpoch {
    pub fn identity(&self) -> &str {
        self.association.identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn start(
        self,
    ) -> Result<
        WorthQueryIteratingWorkflowConvergenceEpoch,
        WorthQueryWorkflowConvergenceStartRejection,
    > {
        match self.association.start() {
            Ok(managed_run) => Ok(WorthQueryIteratingWorkflowConvergenceEpoch {
                association: managed_run,
            }),
            Err(rejection) => Err(WorthQueryWorkflowConvergenceStartRejection {
                association: rejection,
            }),
        }
    }
}

pub struct WorthQueryWorkflowConvergenceStartRejection {
    association: WorkflowStartRejectionAssociation,
}

impl WorthQueryWorkflowConvergenceStartRejection {
    pub fn managed_run_rejection(&self) -> &WorthQueryWorkflowRunStartRejection {
        self.association.managed_run_rejection()
    }

    pub fn into_admitted(self) -> WorthQueryAdmittedWorkflowConvergenceEpoch {
        WorthQueryAdmittedWorkflowConvergenceEpoch {
            association: self.association.into_admitted(),
        }
    }
}

pub struct WorthQueryIteratingWorkflowConvergenceEpoch {
    pub(super) association: WorkflowIteratingEpochAssociation,
}

impl WorthQueryIteratingWorkflowConvergenceEpoch {
    pub fn identity(&self) -> &str {
        self.association.identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.association.logical_run_identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn begin_stage_iteration(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        stage_identity: &str,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<
        WorthQueryStartedWorkflowConvergenceIteration,
        WorthQueryWorkflowConvergenceIterationStartRejection,
    > {
        WorkflowIterationAssociation::begin(phase, self.association, stage_identity, request)
            .map(|association| WorthQueryStartedWorkflowConvergenceIteration { association })
            .map_err(start::admit_start_rejection)
    }
}

pub struct WorthQueryStartedWorkflowConvergenceIteration {
    pub(super) association: WorkflowIterationAssociation<WorthQueryActiveWorkflowGraphExecution>,
}

impl WorthQueryStartedWorkflowConvergenceIteration {
    pub fn epoch_identity(&self) -> &str {
        self.association.epoch_identity()
    }

    pub fn request_cancellation(
        &self,
        reason: worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason,
    ) -> Result<
        worth_runtime_bridge::facade::BridgeManagedExecutionCancellation,
        worth_runtime_bridge::facade::BridgeManagedExecutionInterruptionFailure,
    > {
        self.association.request_cancellation(reason)
    }

    pub fn advance(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorthQueryWorkflowConvergenceStepOutcome {
        admit_associated_step(self.association.advance(phase))
    }

    pub fn abandon(self) -> WorthQueryWorkflowConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub enum WorthQueryWorkflowConvergenceStepOutcome {
    /// Refused before observation; retains the active iteration.
    ForeignAdvancementPhase(WorthQueryStartedWorkflowConvergenceIteration),
    Continue(WorthQueryPausedWorkflowConvergenceIteration),
    ChunkReady(WorthQueryPendingWorkflowConvergenceChunk),
    Completed(WorthQueryWorkflowConvergenceIterationOutcome),
    Terminal(WorthQueryWorkflowConvergenceIterationOutcome),
}

pub struct WorthQueryPausedWorkflowConvergenceIteration {
    pub(super) association: WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution>,
}

impl WorthQueryPausedWorkflowConvergenceIteration {
    pub fn advance(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorthQueryWorkflowConvergenceStepOutcome {
        admit_associated_step(self.association.advance(phase))
    }

    pub fn yield_iteration(self) -> WorthQueryWorkflowConvergenceYieldOutcome {
        yield_transition::admit_associated_yield(self.association.yield_iteration())
    }

    pub fn abandon(self) -> WorthQueryWorkflowConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub struct WorthQueryPendingWorkflowConvergenceChunk {
    pub(super) association: WorkflowIterationAssociation<WorthQueryPendingWorkflowGraphChunk>,
}

impl WorthQueryPendingWorkflowConvergenceChunk {
    pub fn chunk(&self) -> &crate::domain_computation::WorthQueryGraphReadMaterial {
        self.association.chunk()
    }

    pub fn queue_depth(&self) -> u64 {
        self.association.queue_depth()
    }

    pub fn queue_capacity(&self) -> u64 {
        self.association.queue_capacity()
    }

    pub fn acknowledge(self) -> WorthQueryWorkflowConvergenceStepOutcome {
        admit_associated_step(self.association.acknowledge())
    }

    pub fn abandon(self) -> WorthQueryWorkflowConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub(super) fn start_termination(
    denial: super::super::WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryWorkflowConvergenceIterationOutcome,
) -> WorthQueryWorkflowConvergenceIterationStartTermination {
    start::start_termination(denial, outcome)
}

fn admit_associated_step(
    outcome: WorkflowAssociatedStepOutcome,
) -> WorthQueryWorkflowConvergenceStepOutcome {
    match outcome {
        WorkflowAssociatedStepOutcome::ForeignAdvancementPhase(association) => {
            WorthQueryWorkflowConvergenceStepOutcome::ForeignAdvancementPhase(
                WorthQueryStartedWorkflowConvergenceIteration { association },
            )
        }
        WorkflowAssociatedStepOutcome::Continue(association) => {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(
                WorthQueryPausedWorkflowConvergenceIteration { association },
            )
        }
        WorkflowAssociatedStepOutcome::ChunkReady(association) => {
            WorthQueryWorkflowConvergenceStepOutcome::ChunkReady(
                WorthQueryPendingWorkflowConvergenceChunk { association },
            )
        }
        WorkflowAssociatedStepOutcome::Completed(outcome) => {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome)
        }
        WorkflowAssociatedStepOutcome::Terminal(outcome) => {
            WorthQueryWorkflowConvergenceStepOutcome::Terminal(outcome)
        }
    }
}
