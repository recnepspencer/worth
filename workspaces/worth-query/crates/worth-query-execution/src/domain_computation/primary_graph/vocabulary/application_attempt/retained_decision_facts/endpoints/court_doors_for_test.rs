//! Sealed test doors over this module's private members.
use super::*;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn capture_for_test(
        input: &[EntityId],
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::capture(input, policy)
    }
}

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn from_exact_iterator_for_test(
        count: usize,
        input: impl IntoIterator<Item = EntityId>,
        control: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_exact_iterator(count, input, control)
    }
}

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn union_for_test(
        &self,
        other: &Self,
        policy: crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::union(self, other, policy)
    }
}
