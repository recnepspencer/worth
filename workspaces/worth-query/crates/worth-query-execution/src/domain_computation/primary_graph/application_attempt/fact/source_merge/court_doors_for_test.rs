//! Sealed test doors over this module's private members.
use super::*;

impl Fact {
    #[doc(hidden)]
    pub fn merge_same_source_fact_for_test(
        &mut self,
        duplicate: Self,
        control: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<bool, crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial> {
        Self::merge_same_source_fact(self, duplicate, control)
    }
}
