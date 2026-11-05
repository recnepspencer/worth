//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow_transition_program::publication`: the items below and every impl they own.

/// A condition the instance waits on: the published expression and the
/// operands it reads, each an installed query result supplied by its binding.
/// True selects the satisfied successor and false the unsatisfied one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) instance: worth_relational::facade::identity::EntityId,
    pub(in crate::domain_computation::primary_graph) node_path: String,
    pub(in crate::domain_computation::primary_graph) transition_identity: String,
    pub(in crate::domain_computation::primary_graph) occurrence: u64,
    pub(in crate::domain_computation::primary_graph) condition:
        crate::domain_computation::primary_graph::vocabulary::application_attempt::condition::CompiledWorkflowCondition,
}

impl RequiredWorkflowCondition {
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::vocabulary::application_attempt::selection::SelectedWorkflowCondition,
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
    ) -> &[crate::domain_computation::primary_graph::vocabulary::application_attempt::condition::WorkflowConditionOperand]
{
        &self.condition.operands
    }
}
