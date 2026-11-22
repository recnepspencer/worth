//! A descriptive proposal slot, never permission to propose or execute.
use crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowProposal;
use worth_relational::facade::identity::EntityId;

/// The current native node needs a proposal of this declared operation/input.
/// Fresh proposal admission still owns publication; this value grants no authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequiredWorkflowProposal {
    instance: EntityId,
    node_path: String,
    operation: String,
    input_type: String,
    occurrence: u64,
}
impl RequiredWorkflowProposal {
    pub fn from_selected(
        instance: EntityId,
        occurrence: u64,
        selected: SelectedWorkflowProposal<'_>,
    ) -> Self {
        Self {
            instance,
            node_path: selected.node_path.to_owned(),
            operation: selected.operation.to_owned(),
            input_type: selected.input_type.to_owned(),
            occurrence,
        }
    }
    pub const fn instance(&self) -> EntityId {
        self.instance
    }
    pub fn node_path(&self) -> &str {
        &self.node_path
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn input_type(&self) -> &str {
        &self.input_type
    }
    pub const fn occurrence(&self) -> u64 {
        self.occurrence
    }
}
