use worth_query_declaration::facade::application_program::ApplicationProgramRevision;

use super::PreparedWorkflowTransitionReplays;

pub struct PreparedWorkflowCondition<Schema, Operation, Input, Scope> {
    pub admitted:
        crate::domain_computation::primary_graph::workflow::instance::AdmittedWorkflowTransition<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    pub required: RequiredWorkflowCondition,
    pub layout:
        crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    pub program_revision: ApplicationProgramRevision,
    pub replays: PreparedWorkflowTransitionReplays,
}

impl<Schema, Operation, Input, Scope> PreparedWorkflowCondition<Schema, Operation, Input, Scope> {
    pub const fn required(&self) -> &RequiredWorkflowCondition {
        &self.required
    }

    pub fn into_required(self) -> RequiredWorkflowCondition {
        self.required
    }
}

/// A condition the instance waits on: the published expression and the
/// operands it reads, each an installed query result supplied by its binding.
/// True selects the satisfied successor and false the unsatisfied one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredWorkflowCondition {
    pub(crate) instance: worth_relational::facade::identity::EntityId,
    pub(crate) node_path: String,
    pub(crate) transition_identity: String,
    pub(crate) occurrence: u64,
    pub(crate) condition:
        crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowCondition,
}

impl RequiredWorkflowCondition {
    pub(crate) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowCondition,
    ) -> Self {
        Self {
            instance,
            node_path,
            transition_identity,
            occurrence,
            condition: selected.condition,
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
    /// Operands in name order.
    pub fn operands(
        &self,
    ) -> &[crate::domain_computation::primary_graph::workflow::definition::WorkflowConditionOperand]
    {
        &self.condition.operands
    }
}
