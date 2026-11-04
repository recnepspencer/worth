//! Sealed test doors over this module's private members.
use super::*;

impl WorkflowAssessmentEvidenceMeaning {
    #[doc(hidden)]
    pub fn new_for_test(
        identity: String,
        producer: String,
        family: String,
        query: String,
        parameter_type: String,
        result_type: String,
        binding: String,
        subject: EntityId,
        proposal_identity: String,
        coverage_identity: String,
        source_identity: String,
        passing: bool,
        publication_identity: String,
        output_content_identity: String,
        program_revision: String,
        retained_bytes: u64,
        currentness_facts: std::sync::Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    ) -> Self {
        Self {
            identity,
            producer,
            family,
            query,
            parameter_type,
            result_type,
            binding,
            subject,
            proposal_identity,
            coverage_identity,
            source_identity,
            passing,
            publication_identity,
            output_content_identity,
            program_revision,
            retained_bytes,
            currentness_facts,
        }
    }
}
