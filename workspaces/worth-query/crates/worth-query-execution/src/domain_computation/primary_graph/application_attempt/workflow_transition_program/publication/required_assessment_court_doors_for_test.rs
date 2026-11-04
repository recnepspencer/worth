//! Sealed test doors over this module's private members.
use super::*;

impl RequiredWorkflowAssessment {
    #[doc(hidden)]
    pub fn coverage_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.coverage_identity
    }
}

impl RequiredWorkflowAssessment {
    #[doc(hidden)]
    pub fn proposal_identity_mut_for_test(&mut self) -> &mut String {
        &mut self.proposal_identity
    }
}

impl RequiredWorkflowAssessment {
    #[doc(hidden)]
    pub fn new_for_test(
        instance: worth_relational::facade::identity::EntityId,
        node_path: String,
        transition_identity: String,
        occurrence: u64,
        query: String,
        parameter_type: String,
        result_type: String,
        binding: String,
        proposal_identity: String,
        coverage_identity: String,
        program_revision: String,
    ) -> Self {
        Self {
            instance,
            node_path,
            transition_identity,
            occurrence,
            query,
            parameter_type,
            result_type,
            binding,
            proposal_identity,
            coverage_identity,
            program_revision,
        }
    }
}
