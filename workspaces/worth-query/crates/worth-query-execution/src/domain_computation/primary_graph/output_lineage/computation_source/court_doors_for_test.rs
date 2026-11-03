//! Sealed test doors over this module's private members.
use super::*;

impl ComputationSourceEvidence {
    #[doc(hidden)]
    pub fn retain_facts_for_test(
        self, facts: impl Into<
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload,
        >) -> super::super::RetainedSourceFacts {
        Self::retain_facts(self, facts)
    }
}
