use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::PreparedWorkflowTransitionReplay;

pub struct PreparedWorkflowOperation<Schema, Operation, Input, Scope> {
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) required:
        RequiredWorkflowOperation,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) program_revision:
        ApplicationProgramRevision,
    pub(in crate::domain_computation::primary_graph::application_attempt::workflow_transition_program) replays:
        Box<[PreparedWorkflowTransitionReplay]>,
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowOperation<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowOperation {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowOperation {
        self.required
    }
}

#[derive(Clone, Debug)]
pub struct RequiredWorkflowOperation {
    pub(super) instance: worth_relational::facade::identity::EntityId,
    pub(super) node_path: String,
    pub(super) transition_identity: String,
    pub(super) transition_identity_bytes: [u8; 32],
    pub(super) occurrence: u64,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) input_identity: [u8; 32],
}

impl RequiredWorkflowOperation {
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        transition_identity_bytes: [u8; 32],
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowOperation,
        input_identity: [u8; 32],
    ) -> Self {
        Self {
            instance,
            node_path,
            transition_identity,
            transition_identity_bytes,
            occurrence,
            operation: selected.operation,
            input_type: selected.input_type,
            input_identity,
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

    #[doc(hidden)]
    pub const fn transition_identity_bytes(&self) -> &[u8; 32] {
        &self.transition_identity_bytes
    }

    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }

    pub fn operation(&self) -> &str {
        &self.operation
    }

    pub fn input_type(&self) -> &str {
        &self.input_type
    }

    #[doc(hidden)]
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }
}
