//! Sealed test doors over this module's private members.
use super::*;

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn capture_for_test(
        input: &[EntityId],
        policy: super::super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::capture(input, policy)
    }
}

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn from_exact_iterator_for_test(
        count: usize,
        input: impl IntoIterator<Item = EntityId>,
        control: super::super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::from_exact_iterator(count, input, control)
    }
}

impl AdmittedAdjacencyEndpoints {
    #[doc(hidden)]
    pub fn union_for_test(
        &self,
        other: &Self,
        policy: super::super::StorageControl<'_, '_>,
    ) -> Result<Self, StoreDenial> {
        Self::union(self, other, policy)
    }
}
