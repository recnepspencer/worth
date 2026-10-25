use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::residency::{Full, RelationalRegionRetention, RelationalResidencyLease};

/// The authoritative record substrate keyed by partition.
///
/// The map is the spine only: every partition behind it is structurally shared,
/// so copying the map copies pointers, never records.
pub(crate) type PartitionMap = BTreeMap<PartitionId, RelationalRegionRetention>;

/// One immutable, structurally shared edition of the authoritative record
/// substrate.
///
/// Acquiring an edition is a single atomic increment. It copies nothing, it
/// retains no lock, and it can therefore be held across record materialization,
/// re-entrant substrate lookups, derived projection work, and leased packet work
/// without any possibility of deadlocking a settling executor.
///
/// An edition is frozen at the instant it is taken. A writer that runs while an
/// edition is outstanding copies exactly what that edition observes and leaves
/// the edition itself untouched, which is what makes a read-side snapshot
/// structural rather than conventional.
///
/// Holding an edition is therefore not free to everyone: it is free to the
/// reader and it is the writer that pays, once, and only while observed. Read
/// paths must pin exactly one edition for a whole traversal; read-modify-read
/// loops must drop theirs before each write and take a fresh one after.
#[derive(Clone, Debug, Default)]
pub(crate) struct PartitionEdition {
    partitions: Arc<PartitionMap>,
}

impl PartitionEdition {
    pub(crate) fn new(partitions: PartitionMap) -> Self {
        Self {
            partitions: Arc::new(partitions),
        }
    }

    /// Lend one partition's authoritative state for the life of this edition.
    ///
    /// The borrow is tied to the edition handle, not to a lock, so callers keep
    /// the handle alive for the whole traversal rather than copying out of it.
    pub(crate) fn partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.partitions.get(&partition_id).map(AsRef::as_ref)
    }

    /// A counted handle onto one partition, for callers that must outlive the
    /// edition itself.
    pub(crate) fn shared_partition(
        &self,
        partition_id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.partitions
            .get(&partition_id)
            .map(|region| region.lease().clone())
    }

    pub(crate) fn cells(&self) -> impl Iterator<Item = (PartitionId, &RelationalRegionRetention)> {
        self.partitions.iter().map(|(id, region)| (*id, region))
    }

    pub(crate) fn resident_pin_lease(
        &self,
        id: PartitionId,
    ) -> Result<RelationalResidencyLease<Full>, crate::storage::data::RelationalStorageUnavailable>
    {
        self.partitions
            .get(&id)
            .ok_or(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(id),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            })?
            .pin_lease()
    }

    pub(super) fn demand_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        if let Some(region) = self.partitions.get(&id) {
            region.demand_full(execution, 1 << 30)?;
        }
        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.partitions.len()
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (PartitionId, &PartitionState)> {
        self.partitions
            .iter()
            .map(|(partition_id, partition)| (*partition_id, partition.as_ref()))
    }

    pub(crate) fn partitions(&self) -> impl Iterator<Item = &PartitionState> {
        self.partitions.values().map(AsRef::as_ref)
    }

    /// Owned copies of every partition, for checkpoint capture and recovery.
    ///
    /// This is reconstructive-lane work by construction: it materializes the
    /// whole substrate and its callers must charge it accordingly.
    pub(crate) fn materialize_owned_partitions(&self) -> Vec<PartitionState> {
        self.partitions
            .values()
            .map(|partition| partition.as_ref().clone())
            .collect()
    }

    /// Exclusive access to the map spine, copying it only when another edition
    /// of the same map is still outstanding.
    ///
    /// Returns whether the spine had to be copied so the caller can charge that
    /// copy to its declared lane. Never call this without accounting for it.
    pub(super) fn map_mut(&mut self) -> (&mut PartitionMap, bool) {
        let copied = Arc::get_mut(&mut self.partitions).is_none();
        (Arc::make_mut(&mut self.partitions), copied)
    }
}

impl PartitionAccess for PartitionEdition {
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.partition(partition_id)
    }

    fn partition_ids(&self) -> Vec<PartitionId> {
        self.partitions.keys().copied().collect()
    }

    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.partitions.keys().copied())
    }

    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        use crate::storage::overlay::collect_partition_ids;
        collect_partition_ids(
            self.partitions.len(),
            self.partitions.keys().copied(),
            memory,
        )
    }
}
