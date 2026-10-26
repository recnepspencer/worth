mod edition_copy_lane;
mod edition_writer;
mod partition_edition;
mod residency_demand;
pub(crate) use residency_demand::MirrorResidencyDemandBinding;

pub(crate) use edition_copy_lane::PartitionEditionCopyLane;
pub(crate) use edition_writer::{
    PartitionEditionWriter, ResidentPartitionEditionWriter, ResidentPartitionEntry,
};
pub(crate) use partition_edition::{PartitionEdition, PartitionMap, ResidentPartitionEdition};

use crate::identity::data::PartitionId;
use crate::runtime::state::subsystems::{
    RuntimeInstrumentation, RuntimeOwnedState, RuntimeSubsystem,
};
use crate::storage::overlay::PartitionState;
use crate::storage::residency::{Full, RelationalRegionRetention, RelationalResidencyLease};

/// The runtime's record substrate, owned behind its own lock.
///
/// Readers never retain the lock and never copy the substrate: `acquire`
/// returns a structurally shared edition after a single atomic increment and
/// releases the lock immediately, so no read path can deadlock against a
/// settling executor and no lock is ever held across derived work.
///
/// Writers take the lock exclusively and copy exactly what is observed: the map
/// spine once per guard if a reader edition is outstanding, and an individual
/// partition only when a reader still holds the previous edition of it.
#[derive(Debug, Default)]
pub(crate) struct StorageSubsystem {
    partitions: RuntimeOwnedState<PartitionEdition>,
}

impl StorageSubsystem {
    pub(crate) fn install_cells(&self, cells: PartitionMap) {
        *self.partitions.write() = PartitionEdition::new(cells);
    }
    /// One structurally shared edition of the substrate. O(1): one atomic
    /// increment, no allocation, no retained lock.
    pub(crate) fn acquire(&self) -> PartitionEdition {
        self.partitions.read().clone()
    }

    /// Exclusive authority to install the next edition, charged to `lane`.
    pub(crate) fn edit<'subsystem>(
        &'subsystem self,
        instrumentation: &'subsystem RuntimeInstrumentation,
        lane: PartitionEditionCopyLane,
    ) -> PartitionEditionWriter<'subsystem> {
        PartitionEditionWriter::new(self.partitions.write(), instrumentation, lane)
    }

    /// Demand outside the write lock, then seal the current edition. Publication
    /// installs resident entries; only an exclusive recovery can install cold ones.
    pub(crate) fn demand_resident_edit<'subsystem>(
        &'subsystem self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        instrumentation: &'subsystem RuntimeInstrumentation,
    ) -> Result<
        ResidentPartitionEditionWriter<'subsystem>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        loop {
            self.acquire().demand_all(execution)?;
            match self
                .edit(instrumentation, PartitionEditionCopyLane::Ordinary)
                .accept_resident()
            {
                Ok(writer) => return Ok(writer),
                Err(writer) => drop(writer),
            }
        }
    }

    pub(crate) fn partition_ids(&self) -> Vec<PartitionId> {
        use crate::storage::overlay::CurrentPartitionAccess;
        self.partitions.read().partition_ids()
    }

    pub(crate) fn partition(
        &self,
        partition_id: PartitionId,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.partitions.read().shared_partition(partition_id)
    }

    pub(crate) fn partition_count(&self) -> usize {
        self.partitions.read().len()
    }

    /// Owned copies of every partition, for checkpoint capture and recovery.
    ///
    /// Reconstructive-lane work; the caller supplies the accounting sink.
    pub(crate) fn materialize_owned_partitions(
        &self,
        instrumentation: &RuntimeInstrumentation,
    ) -> Vec<PartitionState> {
        let edition = self.acquire();
        let partitions = edition.materialize_owned_partitions();
        PartitionEditionCopyLane::Reconstructive
            .charge_partition_materialization(instrumentation, partitions.len());
        partitions
    }

    pub(crate) fn install_owned(
        &self,
        instrumentation: &RuntimeInstrumentation,
        lane: PartitionEditionCopyLane,
        partitions: impl IntoIterator<Item = (PartitionId, PartitionState)>,
    ) {
        let installed: PartitionMap = partitions
            .into_iter()
            .map(|(partition_id, partition)| {
                (partition_id, RelationalRegionRetention::resident(partition))
            })
            .collect();
        self.edit(instrumentation, lane).install(installed);
    }
}

impl RuntimeSubsystem for StorageSubsystem {
    type Config = ();

    fn new(_: &Self::Config) -> Self {
        Self::default()
    }

    /// A detached deep copy that shares no authority with its origin.
    ///
    /// Every partition is materialized so the fork's pin counters start clean,
    /// which is whole-state reconstructive work; the fork site charges it.
    fn fork(&self) -> Self {
        let mut partitions = self.acquire().cloned_map();
        for partition in partitions.values_mut() {
            partition.make_mut().clear_runtime_pin_counters();
        }
        Self {
            partitions: RuntimeOwnedState::new(PartitionEdition::new(partitions)),
        }
    }
}
