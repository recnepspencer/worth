use crate::domain_computation::primary_graph::workflow::instance::{
    WorkflowTransitionReplayProjection, WorkflowTransitionReplayRetention,
};

#[derive(Default)]
#[doc(hidden)]
pub struct PreparedWorkflowTransitionReplays {
    retained: WorkflowTransitionReplayRetention,
    pub(super) probe_identity: Option<[u8; 32]>,
}

impl PreparedWorkflowTransitionReplays {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn retained(
        retained: WorkflowTransitionReplayRetention,
    ) -> Self {
        Self {
            retained,
            probe_identity: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn with_probe_identity(
        mut self,
        identity: [u8; 32],
    ) -> Self {
        self.probe_identity = Some(identity);
        self
    }

    pub(super) fn materialize(&self) -> Box<[WorkflowTransitionReplayProjection]> {
        self.retained.materialize()
    }
}
