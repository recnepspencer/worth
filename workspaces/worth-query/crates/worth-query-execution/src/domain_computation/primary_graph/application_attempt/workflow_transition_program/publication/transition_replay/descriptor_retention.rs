use worth_relational::facade::identity::VersionId;

use crate::domain_computation::primary_graph::{
    workflow::instance::{WorkflowInstanceProgressKey, WorkflowTransitionReplayProjection},
    WorthQueryPrimaryGraphApplicationRuntime,
};

#[derive(Default)]
#[doc(hidden)]
pub struct PreparedWorkflowTransitionReplays {
    retention: Option<(WorkflowInstanceProgressKey, Option<VersionId>)>,
    pub(super) probe_identity: Option<[u8; 32]>,
    cold: Box<[WorkflowTransitionReplayProjection]>,
}

impl PreparedWorkflowTransitionReplays {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn retained(
        retention: (WorkflowInstanceProgressKey, Option<VersionId>),
        cold: Box<[WorkflowTransitionReplayProjection]>,
    ) -> Self {
        Self {
            retention: Some(retention),
            probe_identity: None,
            cold,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) fn with_probe_identity(
        mut self,
        identity: [u8; 32],
    ) -> Self {
        self.probe_identity = Some(identity);
        self
    }

    pub(super) fn materialize<Schema>(
        &self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Box<[WorkflowTransitionReplayProjection]> {
        self.retention
            .and_then(|(key, revision)| {
                runtime
                    .primary_provider
                    .graph
                    .with_workflow_instance_progress_mut(key, |retention| {
                        retention.replays(key, revision)
                    })
            })
            .unwrap_or_else(|| self.cold.clone())
    }
}
