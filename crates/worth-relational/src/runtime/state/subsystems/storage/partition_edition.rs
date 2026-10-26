use std::collections::BTreeMap;
use std::sync::Arc;

use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::residency::{Full, RelationalRegionRetention, RelationalResidencyLease};

mod resident;
mod symbol_read;
pub(crate) use resident::ResidentPartitionEdition;

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
    pub(crate) fn with_symbol_admission<'read>(
        &'read self,
        symbols: &'read crate::runtime::RuntimeSymbolTable,
    ) -> impl PartitionAccess + 'read {
        symbol_read::SymbolAdmittedEdition {
            edition: self,
            symbols,
        }
    }

    pub(crate) fn demand_partition_with_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_partition(execution, id)?;
        if let Some(region) = self.partitions.get(&id) {
            region.lease().admit_symbols(execution, symbols)?;
        }
        Ok(())
    }

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

    pub(crate) fn admitted_full_lease(
        &self,
        id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.partitions.get(&id)?.admitted_full_lease()
    }

    pub(crate) fn resident_head_lease(
        &self,
        id: PartitionId,
    ) -> Result<
        RelationalResidencyLease<crate::storage::residency::Head>,
        crate::storage::data::RelationalStorageUnavailable,
    > {
        self.partitions
            .get(&id)
            .ok_or(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(id),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            })?
            .pin_head_lease()
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

    pub(crate) fn demand_partition_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<
        Option<RelationalResidencyLease<crate::storage::residency::Head>>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        self.partitions
            .get(&id)
            .map(|region| region.demand_head(execution, 1 << 30))
            .transpose()
    }

    /// A scan declares the whole immutable edition before visiting any row.
    pub(crate) fn demand_all(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<ResidentPartitionEdition, crate::storage::residency::RelationalResidencyDemandFailure>
    {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        for region in self.partitions.values() {
            region.demand_full(execution, 1 << 30)?;
            region.lease().admit_symbols(execution, symbols)?;
        }
        Ok(ResidentPartitionEdition::new(self.clone()))
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
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        use crate::storage::residency::PartitionProbe;
        match self.partitions.get(&id) {
            None => PartitionProbe::Absent,
            Some(region) => match region.admitted_partition() {
                Some(partition) => PartitionProbe::Resident(partition),
                None => PartitionProbe::Cold,
            },
        }
    }

    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.demand_partition(execution, id)
    }
    fn get_partition(&self, partition_id: PartitionId) -> Option<&PartitionState> {
        self.partition(partition_id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for PartitionEdition {
    fn get_current_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.partitions
            .get(&id)
            .and_then(|region| region.admitted_head_view())
    }
    fn probe_head_partition(
        &self,
        id: crate::identity::data::PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        use crate::storage::residency::PartitionHeadProbe as Probe;
        match self.partitions.get(&id) {
            None => Probe::Absent,
            Some(region) => region
                .admitted_head_view()
                .map_or(Probe::Cold, Probe::Resident),
        }
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: crate::identity::data::PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(crate::storage::residency::RelationalResidencyDemandFailure::Request)?;
        if let Some(region) = self.partitions.get(&id) {
            region.demand_head_cached(execution, 1 << 30)?;
        }
        Ok(())
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
