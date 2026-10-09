//! Workflow managed step transitions over one sealed association.

use super::WorkflowIterationAssociation;
use crate::domain_computation::{
    WorthQueryActiveWorkflowGraphExecution, WorthQueryPausedWorkflowGraphExecution,
    WorthQueryPendingWorkflowGraphChunk, WorthQueryWorkflowGraphStepOutcome,
};

pub(in super::super) enum WorkflowAssociatedStepOutcome {
    ForeignAdvancementPhase(WorkflowIterationAssociation<WorthQueryActiveWorkflowGraphExecution>),
    Continue(WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution>),
    ChunkReady(WorkflowIterationAssociation<WorthQueryPendingWorkflowGraphChunk>),
    Completed(super::super::WorthQueryWorkflowConvergenceIterationOutcome),
    Terminal(super::super::WorthQueryWorkflowConvergenceIterationOutcome),
}

impl WorkflowIterationAssociation<WorthQueryActiveWorkflowGraphExecution> {
    pub(in super::super) fn advance(
        self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorkflowAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_step(
            core,
            graph,
            provider,
            stage_identity,
            managed.advance(execution),
        )
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_terminal(core, graph, provider, stage_identity, managed.abandon())
    }
}

impl WorkflowIterationAssociation<WorthQueryPausedWorkflowGraphExecution> {
    pub(in super::super) fn advance(
        self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> WorkflowAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_step(
            core,
            graph,
            provider,
            stage_identity,
            managed.advance(execution),
        )
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_terminal(core, graph, provider, stage_identity, managed.abandon())
    }
}

impl WorkflowIterationAssociation<WorthQueryPendingWorkflowGraphChunk> {
    pub(in super::super) fn chunk(
        &self,
    ) -> &crate::domain_computation::WorthQueryGraphReadMaterial {
        self.managed.chunk()
    }

    pub(in super::super) fn queue_depth(&self) -> u64 {
        self.managed.queue_depth()
    }

    pub(in super::super) fn queue_capacity(&self) -> u64 {
        self.managed.queue_capacity()
    }

    pub(in super::super) fn acknowledge(self) -> WorkflowAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_step(core, graph, provider, stage_identity, managed.acknowledge())
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            stage_identity,
            managed,
        } = self;
        admit_terminal(core, graph, provider, stage_identity, managed.abandon())
    }
}

fn admit_step(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: std::sync::Arc<dyn super::super::super::super::WorthQueryConvergenceDomainProvider>,
    stage_identity: std::sync::Arc<str>,
    outcome: WorthQueryWorkflowGraphStepOutcome,
) -> WorkflowAssociatedStepOutcome {
    match outcome {
        WorthQueryWorkflowGraphStepOutcome::ForeignAdvancementPhase(managed) => {
            WorkflowAssociatedStepOutcome::ForeignAdvancementPhase(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            })
        }
        WorthQueryWorkflowGraphStepOutcome::Continue(managed) => {
            WorkflowAssociatedStepOutcome::Continue(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            })
        }
        WorthQueryWorkflowGraphStepOutcome::ChunkReady(managed) => {
            WorkflowAssociatedStepOutcome::ChunkReady(WorkflowIterationAssociation {
                core,
                graph,
                provider,
                stage_identity,
                managed,
            })
        }
        WorthQueryWorkflowGraphStepOutcome::Completed(managed) => {
            WorkflowAssociatedStepOutcome::Completed(
                WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                }
                .admit_completion(),
            )
        }
        WorthQueryWorkflowGraphStepOutcome::Cancelled(managed)
        | WorthQueryWorkflowGraphStepOutcome::TimedOut(managed)
        | WorthQueryWorkflowGraphStepOutcome::Exhausted(managed)
        | WorthQueryWorkflowGraphStepOutcome::Degraded(managed)
        | WorthQueryWorkflowGraphStepOutcome::Failed(managed) => {
            WorkflowAssociatedStepOutcome::Terminal(
                WorkflowIterationAssociation {
                    core,
                    graph,
                    provider,
                    stage_identity,
                    managed,
                }
                .admit_terminal(),
            )
        }
    }
}

fn admit_terminal(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: std::sync::Arc<dyn super::super::super::super::WorthQueryConvergenceDomainProvider>,
    stage_identity: std::sync::Arc<str>,
    outcome: WorthQueryWorkflowGraphStepOutcome,
) -> super::super::WorthQueryWorkflowConvergenceIterationOutcome {
    match admit_step(core, graph, provider, stage_identity, outcome) {
        WorkflowAssociatedStepOutcome::Terminal(outcome) => outcome,
        _ => unreachable!("managed abandon must terminalize its exact workflow association"),
    }
}
