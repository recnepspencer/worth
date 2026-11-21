use crate::domain_computation::primary_graph::workflow::instance::{
    WorkflowTransitionReplayProjection, WorkflowTransitionReplayRetention,
};

#[derive(Clone, Default)]
#[doc(hidden)]
pub struct PreparedWorkflowTransitionReplays {
    pub retained: WorkflowTransitionReplayRetention,
    pub probe_identity: Option<[u8; 32]>,
    pub navigation_back: bool,
}

impl PreparedWorkflowTransitionReplays {
    pub fn retained(
        retained: WorkflowTransitionReplayRetention,
    ) -> Self {
        Self {
            retained,
            probe_identity: None,
            navigation_back: false,
        }
    }

    pub fn for_navigation_back(
        mut self,
    ) -> Self {
        self.navigation_back = true;
        self
    }

    pub fn with_probe_identity(
        mut self,
        identity: [u8; 32],
    ) -> Self {
        if !self.navigation_back {
            self.probe_identity = Some(identity);
        }
        self
    }

    pub fn materialize(&self) -> Box<[WorkflowTransitionReplayProjection]> {
        self.retained
            .materialize()
            .into_vec()
            .into_iter()
            .filter(|replay| replay.navigation_back == self.navigation_back)
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }
}
