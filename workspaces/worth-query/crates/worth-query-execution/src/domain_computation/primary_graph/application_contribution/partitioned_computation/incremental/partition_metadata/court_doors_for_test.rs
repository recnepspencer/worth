//! Sealed test doors over this module's private members.
use super::*;

impl<Key> PartitionMetadata<Key> {
    #[doc(hidden)]
    pub fn from_recorded_for_test(
        entries:BTreeMap<PartitionIdentity,(Arc<Key>,crate::domain_computation::primary_graph::invariant_projection::ComputationCallCharge)>,
        policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Self, ExecutionOrderedMapDenial> {
        Self::from_recorded(entries, policy)
    }
}
