use super::super::WorthQueryConvergenceEpochCounters;
use super::association::{
    DirectAdmittedEpochAssociation, DirectAssociatedStepOutcome, DirectIteratingEpochAssociation,
    DirectIterationAssociation,
};
use super::completion::WorthQueryDirectConvergenceIterationOutcome;
use super::start::{
    WorthQueryDirectConvergenceIterationStartRejection,
    WorthQueryDirectConvergenceIterationStartTermination,
};
use super::yield_transition::WorthQueryDirectConvergenceYieldOutcome;
use super::{start, yield_transition};
use crate::domain_computation::{
    WorthQueryActiveDirectGraphExecution, WorthQueryManagedGraphCallRequest,
    WorthQueryPausedDirectGraphExecution, WorthQueryPendingDirectGraphChunk,
};

pub struct WorthQueryAdmittedDirectConvergenceEpoch {
    pub(super) association: DirectAdmittedEpochAssociation,
}

impl WorthQueryAdmittedDirectConvergenceEpoch {
    pub fn identity(&self) -> &str {
        self.association.identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn start(self) -> WorthQueryIteratingDirectConvergenceEpoch {
        WorthQueryIteratingDirectConvergenceEpoch {
            association: self.association.start(),
        }
    }
}

pub struct WorthQueryIteratingDirectConvergenceEpoch {
    pub(super) association: DirectIteratingEpochAssociation,
}

impl WorthQueryIteratingDirectConvergenceEpoch {
    pub fn identity(&self) -> &str {
        self.association.identity()
    }

    pub fn logical_run_identity(&self) -> &str {
        self.association.logical_run_identity()
    }

    pub fn counters(&self) -> &WorthQueryConvergenceEpochCounters {
        self.association.counters()
    }

    pub fn begin_iteration(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        request: WorthQueryManagedGraphCallRequest,
    ) -> Result<
        WorthQueryStartedDirectConvergenceIteration,
        WorthQueryDirectConvergenceIterationStartRejection,
    > {
        DirectIterationAssociation::begin(phase, self.association, request)
            .map(|association| WorthQueryStartedDirectConvergenceIteration { association })
            .map_err(start::admit_start_rejection)
    }
}

pub struct WorthQueryStartedDirectConvergenceIteration {
    pub(super) association: DirectIterationAssociation<WorthQueryActiveDirectGraphExecution>,
}

impl WorthQueryStartedDirectConvergenceIteration {
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
    ) -> WorthQueryDirectConvergenceStepOutcome {
        admit_associated_step(self.association.advance(phase))
    }

    pub fn abandon(self) -> WorthQueryDirectConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub enum WorthQueryDirectConvergenceStepOutcome {
    /// Refused before observation; retains the active iteration.
    ForeignAdvancementPhase(WorthQueryStartedDirectConvergenceIteration),
    Continue(WorthQueryPausedDirectConvergenceIteration),
    ChunkReady(WorthQueryPendingDirectConvergenceChunk),
    Completed(WorthQueryDirectConvergenceIterationOutcome),
    Terminal(WorthQueryDirectConvergenceIterationOutcome),
}

pub struct WorthQueryPausedDirectConvergenceIteration {
    pub(super) association: DirectIterationAssociation<WorthQueryPausedDirectGraphExecution>,
}

impl WorthQueryPausedDirectConvergenceIteration {
    pub fn advance(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorthQueryDirectConvergenceStepOutcome {
        admit_associated_step(self.association.advance(phase))
    }

    pub fn yield_iteration(self) -> WorthQueryDirectConvergenceYieldOutcome {
        yield_transition::admit_associated_yield(self.association.yield_iteration())
    }

    pub fn abandon(self) -> WorthQueryDirectConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub struct WorthQueryPendingDirectConvergenceChunk {
    pub(super) association: DirectIterationAssociation<WorthQueryPendingDirectGraphChunk>,
}

impl WorthQueryPendingDirectConvergenceChunk {
    pub fn chunk(&self) -> &crate::domain_computation::WorthQueryGraphReadMaterial {
        self.association.chunk()
    }

    pub fn queue_depth(&self) -> u64 {
        self.association.queue_depth()
    }

    pub fn queue_capacity(&self) -> u64 {
        self.association.queue_capacity()
    }

    pub fn acknowledge(self) -> WorthQueryDirectConvergenceStepOutcome {
        admit_associated_step(self.association.acknowledge())
    }

    pub fn abandon(self) -> WorthQueryDirectConvergenceIterationOutcome {
        self.association.abandon()
    }
}

pub(super) fn start_termination(
    denial: super::super::WorthQueryConvergenceEpochDenial,
    outcome: WorthQueryDirectConvergenceIterationOutcome,
) -> WorthQueryDirectConvergenceIterationStartTermination {
    start::start_termination(denial, outcome)
}

fn admit_associated_step(
    outcome: DirectAssociatedStepOutcome,
) -> WorthQueryDirectConvergenceStepOutcome {
    match outcome {
        DirectAssociatedStepOutcome::ForeignAdvancementPhase(association) => {
            WorthQueryDirectConvergenceStepOutcome::ForeignAdvancementPhase(
                WorthQueryStartedDirectConvergenceIteration { association },
            )
        }
        DirectAssociatedStepOutcome::Continue(association) => {
            WorthQueryDirectConvergenceStepOutcome::Continue(
                WorthQueryPausedDirectConvergenceIteration { association },
            )
        }
        DirectAssociatedStepOutcome::ChunkReady(association) => {
            WorthQueryDirectConvergenceStepOutcome::ChunkReady(
                WorthQueryPendingDirectConvergenceChunk { association },
            )
        }
        DirectAssociatedStepOutcome::Completed(outcome) => {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome)
        }
        DirectAssociatedStepOutcome::Terminal(outcome) => {
            WorthQueryDirectConvergenceStepOutcome::Terminal(outcome)
        }
    }
}
