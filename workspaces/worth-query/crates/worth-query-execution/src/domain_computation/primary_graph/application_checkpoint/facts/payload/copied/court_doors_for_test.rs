//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl FactPayload {
    #[doc(hidden)]
    pub fn copy_funded_for_test(
        facts: &(impl FactSequence + ?Sized),
        control: StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::copy_funded(facts, control)
    }
}
