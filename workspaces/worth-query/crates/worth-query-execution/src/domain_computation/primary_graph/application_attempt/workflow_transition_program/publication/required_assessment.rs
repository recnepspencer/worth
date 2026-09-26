#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub(super) program_revision: String,
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
        program_revision: String,
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
            program_revision,
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
    /// The program revision the evidence is collected under.
    pub fn program_revision(&self) -> &str {
        &self.program_revision
    }
}

#[cfg(test)]
mod tests {
    use worth_relational::facade::identity::{EntityId, PartitionId};

    use super::RequiredWorkflowAssessment;

    #[test]
    fn exact_requirement_distinguishes_proposal_and_coverage_with_same_execution_affinity() {
        let original = RequiredWorkflowAssessment {
            instance: EntityId::new(PartitionId::main(), 1, 1),
            node_path: "review".to_owned(),
            transition_identity: "transition".to_owned(),
            occurrence: 1,
            query: "query".to_owned(),
            parameter_type: "parameters".to_owned(),
            result_type: "result".to_owned(),
            binding: "binding".to_owned(),
            proposal_identity: "proposal-a".to_owned(),
            coverage_identity: "coverage-a".to_owned(),
            program_revision: "revision-a".to_owned(),
        };
        let mut revised_proposal = original.clone();
        revised_proposal.proposal_identity = "proposal-b".to_owned();
        assert_ne!(original, revised_proposal);
        let mut revised_coverage = original.clone();
        revised_coverage.coverage_identity = "coverage-b".to_owned();
        assert_ne!(original, revised_coverage);
    }
}
