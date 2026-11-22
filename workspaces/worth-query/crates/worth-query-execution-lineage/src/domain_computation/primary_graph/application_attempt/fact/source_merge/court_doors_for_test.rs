//! Sealed test doors over this module's private members.
use super::*;

impl Fact {
    #[doc(hidden)]
    pub fn merge_same_source_fact_for_test(
        &mut self,
        duplicate: Self,
        control: super::super::super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<bool, super::super::super::retained_decision_facts::StoreDenial> {
        Self::merge_same_source_fact(self, duplicate, control)
    }
}
