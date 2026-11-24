//! Full reads own the exact leases admitted for the selected edition.
use super::{Full, PartitionAccess, PartitionId, PartitionState, RelationalResidencyLease};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct ResidentPartitionEdition {
    // Keep the selected spine observed as well as its admitted payloads.
    _edition: super::PartitionEdition,
    leases: BTreeMap<PartitionId, RelationalResidencyLease<Full>>,
}

impl ResidentPartitionEdition {
    pub(super) fn new(
        edition: super::PartitionEdition,
        leases: BTreeMap<PartitionId, RelationalResidencyLease<Full>>,
    ) -> Self {
        Self {
            _edition: edition,
            leases,
        }
    }
    pub(crate) fn resident_pin_lease(
        &self,
        id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.leases.get(&id).cloned()
    }
    pub fn partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.leases
            .get(&id)
            .map(|lease| lease.full_view().partition())
    }
    pub fn entries(&self) -> impl Iterator<Item = (PartitionId, &PartitionState)> {
        self.leases
            .iter()
            .map(|(id, lease)| (*id, lease.full_view().partition()))
    }
    pub fn partitions(&self) -> impl Iterator<Item = &PartitionState> {
        self.leases
            .values()
            .map(|lease| lease.full_view().partition())
    }
}
impl PartitionAccess for ResidentPartitionEdition {
    fn get_partition(&self, id: PartitionId) -> Option<&PartitionState> {
        self.partition(id)
    }
}
impl crate::storage::overlay::CurrentPartitionAccess for ResidentPartitionEdition {
    fn get_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.partition(id)
            .map(crate::storage::residency::RelationalRegionHeadView::from_owned_partition)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.leases.keys().copied().collect()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.leases.keys().copied())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        crate::storage::overlay::collect_partition_ids(
            self.leases.len(),
            self.leases.keys().copied(),
            memory,
        )
    }
}

/// Receiver-scoped fixtures for the Relational unit courts.
#[doc(hidden)]
pub trait ResidentPartitionEditionFixturePort: Sized {
    fn resident_pin_lease_for_test(
        &self,
        id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>>;
}

impl ResidentPartitionEditionFixturePort for ResidentPartitionEdition {
    fn resident_pin_lease_for_test(
        &self,
        id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.resident_pin_lease(id)
    }
}
