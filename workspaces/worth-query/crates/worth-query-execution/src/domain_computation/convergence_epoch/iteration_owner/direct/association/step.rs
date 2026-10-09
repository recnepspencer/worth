//! Direct managed step transitions over one sealed association.

use super::DirectIterationAssociation;
use crate::domain_computation::{
    WorthQueryActiveDirectGraphExecution, WorthQueryDirectGraphStepOutcome,
    WorthQueryPausedDirectGraphExecution, WorthQueryPendingDirectGraphChunk,
};

pub(in super::super) enum DirectAssociatedStepOutcome {
    ForeignAdvancementPhase(DirectIterationAssociation<WorthQueryActiveDirectGraphExecution>),
    Continue(DirectIterationAssociation<WorthQueryPausedDirectGraphExecution>),
    ChunkReady(DirectIterationAssociation<WorthQueryPendingDirectGraphChunk>),
    Completed(super::super::WorthQueryDirectConvergenceIterationOutcome),
    Terminal(super::super::WorthQueryDirectConvergenceIterationOutcome),
}

impl DirectIterationAssociation<WorthQueryActiveDirectGraphExecution> {
    pub(in super::super) fn advance(
        self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> DirectAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_step_outcome(core, graph, provider, managed.advance(execution))
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_terminal_outcome(core, graph, provider, managed.abandon())
    }
}

impl DirectIterationAssociation<WorthQueryPausedDirectGraphExecution> {
    pub(in super::super) fn advance(
        self,
        execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    ) -> DirectAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_step_outcome(core, graph, provider, managed.advance(execution))
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_terminal_outcome(core, graph, provider, managed.abandon())
    }
}

impl DirectIterationAssociation<WorthQueryPendingDirectGraphChunk> {
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

    pub(in super::super) fn acknowledge(self) -> DirectAssociatedStepOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_step_outcome(core, graph, provider, managed.acknowledge())
    }

    pub(in super::super) fn abandon(
        self,
    ) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
        let Self {
            core,
            graph,
            provider,
            managed,
        } = self;
        admit_terminal_outcome(core, graph, provider, managed.abandon())
    }
}

fn admit_step_outcome(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: std::sync::Arc<dyn super::super::super::super::WorthQueryConvergenceDomainProvider>,
    outcome: WorthQueryDirectGraphStepOutcome,
) -> DirectAssociatedStepOutcome {
    match outcome {
        WorthQueryDirectGraphStepOutcome::ForeignAdvancementPhase(managed) => {
            DirectAssociatedStepOutcome::ForeignAdvancementPhase(DirectIterationAssociation {
                core,
                graph,
                provider,
                managed,
            })
        }
        WorthQueryDirectGraphStepOutcome::Continue(managed) => {
            DirectAssociatedStepOutcome::Continue(DirectIterationAssociation {
                core,
                graph,
                provider,
                managed,
            })
        }
        WorthQueryDirectGraphStepOutcome::ChunkReady(managed) => {
            DirectAssociatedStepOutcome::ChunkReady(DirectIterationAssociation {
                core,
                graph,
                provider,
                managed,
            })
        }
        WorthQueryDirectGraphStepOutcome::Completed(managed) => {
            DirectAssociatedStepOutcome::Completed(
                DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                }
                .admit_completion(),
            )
        }
        WorthQueryDirectGraphStepOutcome::Cancelled(managed)
        | WorthQueryDirectGraphStepOutcome::TimedOut(managed)
        | WorthQueryDirectGraphStepOutcome::Exhausted(managed)
        | WorthQueryDirectGraphStepOutcome::Degraded(managed)
        | WorthQueryDirectGraphStepOutcome::Failed(managed) => {
            DirectAssociatedStepOutcome::Terminal(
                DirectIterationAssociation {
                    core,
                    graph,
                    provider,
                    managed,
                }
                .admit_terminal(),
            )
        }
    }
}

fn admit_terminal_outcome(
    core: super::super::super::core::WorthQueryConvergenceEpochCore,
    graph: worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority,
    provider: std::sync::Arc<dyn super::super::super::super::WorthQueryConvergenceDomainProvider>,
    outcome: WorthQueryDirectGraphStepOutcome,
) -> super::super::WorthQueryDirectConvergenceIterationOutcome {
    match admit_step_outcome(core, graph, provider, outcome) {
        DirectAssociatedStepOutcome::Terminal(outcome) => outcome,
        _ => unreachable!("managed abandon must terminalize its exact association"),
    }
}
