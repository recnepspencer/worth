use crate::domain_computation::primary_graph::workflow::instance::{
    SelectedWorkflowApproval, SelectedWorkflowTransition,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkflowApprovalDecision {
    Approve,
    Reject,
}

impl WorkflowApprovalDecision {
    pub(in crate::domain_computation::primary_graph) const fn outcome(
        self,
    ) -> worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome
    {
        match self {
            Self::Approve => worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Approved,
            Self::Reject => worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Rejected,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredWorkflowApproval {
    instance: worth_relational::facade::identity::EntityId,
    node_path: String,
    transition_identity: String,
    occurrence: u64,
    capability: String,
    capability_type: String,
    operation: String,
    installed_capability_identity: String,
    target_operation: String,
}

impl RequiredWorkflowApproval {
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        selected: &SelectedWorkflowTransition,
        approval: SelectedWorkflowApproval,
    ) -> Self {
        Self {
            instance,
            node_path: selected.node_path().to_owned(),
            transition_identity: selected.identity().to_owned(),
            occurrence: selected.occurrence(),
            capability: approval.capability,
            capability_type: approval.capability_type,
            operation: approval.operation,
            installed_capability_identity: approval.installed_capability_identity,
            target_operation: approval.target_operation,
        }
    }

    pub const fn instance(&self) -> worth_relational::facade::identity::EntityId {
        self.instance
    }

    pub fn node_path(&self) -> &str {
        &self.node_path
    }

    pub fn transition_identity(&self) -> &str {
        &self.transition_identity
    }

    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub fn capability(&self) -> &str {
        &self.capability
    }

    pub fn capability_type(&self) -> &str {
        &self.capability_type
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn installed_capability_identity(&self) -> &str {
        &self.installed_capability_identity
    }

    pub fn target_operation(&self) -> &str {
        &self.target_operation
    }
}
