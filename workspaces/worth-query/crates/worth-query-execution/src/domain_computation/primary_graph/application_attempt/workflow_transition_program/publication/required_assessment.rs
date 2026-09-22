#[derive(Clone, Debug)]
pub struct RequiredWorkflowAssessment {
    pub(super) instance: worth_relational::facade::identity::EntityId,
    pub(super) node_path: String,
    pub(super) transition_identity: String,
    pub(super) occurrence: u64,
    pub(super) query: String,
    pub(super) parameter_type: String,
    pub(super) result_type: String,
    pub(super) binding: String,
    pub(super) proposal_identity: String,
    pub(super) coverage_identity: String,
}

impl RequiredWorkflowAssessment {
    pub(in crate::domain_computation::primary_graph) fn from_selected(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        selected: crate::domain_computation::primary_graph::workflow::instance::SelectedWorkflowAssessment,
        proposal_identity: String,
        coverage_identity: String,
    ) -> Self {
        Self {
            instance,
            node_path,
            transition_identity,
            occurrence,
            query: selected.query,
            parameter_type: selected.parameter_type,
            result_type: selected.result_type,
            binding: selected.binding,
            proposal_identity,
            coverage_identity,
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
    pub fn query(&self) -> &str {
        &self.query
    }
    pub fn parameter_type(&self) -> &str {
        &self.parameter_type
    }
    pub fn result_type(&self) -> &str {
        &self.result_type
    }
    pub fn binding(&self) -> &str {
        &self.binding
    }
    pub fn proposal_identity(&self) -> &str {
        &self.proposal_identity
    }
    pub fn coverage_identity(&self) -> &str {
        &self.coverage_identity
    }
}
