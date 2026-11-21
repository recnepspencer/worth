use worth_execution::ExecutionArray;

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_discovery::workflow::{
    denial::WorthQueryWorkflowReadDenial, WorthQueryWorkflowReadBasis,
    WorthQueryWorkflowRecordedActor,
};

/// Recorded roles of one transition at the selected observation.
pub struct WorthQueryWorkflowTransitionActors {
    pub(crate) advancer: WorthQueryWorkflowRecordedActor,
    pub(crate) executor: Option<WorthQueryWorkflowRecordedActor>,
}

impl WorthQueryWorkflowTransitionActors {
    pub(crate) fn observed(
        advancer: WorthQueryWorkflowRecordedActor,
        executor: Option<WorthQueryWorkflowRecordedActor>,
    ) -> Self {
        Self { advancer, executor }
    }

    pub fn advancer(&self) -> WorthQueryWorkflowRecordedActor {
        self.advancer
    }

    pub fn executor(&self) -> Option<WorthQueryWorkflowRecordedActor> {
        self.executor
    }
}

/// Independent audit may withhold identities without hiding lifecycle state.
pub enum WorthQueryWorkflowAudit {
    Withheld,
    Visible {
        start: WorthQueryWorkflowRecordedActor,
        end: Option<worth_foundational::facade::WorkflowEndingActorProvenance>,
        transitions: ExecutionArray<WorthQueryWorkflowTransitionActors>,
    },
}

/// Recorded instance state, independent of whether a command is currently legal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryWorkflowObservedState {
    Ready,
    Completed,
    Cancelled,
    Migrated,
    Terminated,
}
