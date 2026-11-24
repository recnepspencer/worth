pub mod edition_copy_lane;
pub mod edition_writer;
pub mod mirror_state;
pub mod partition_edition;
pub mod residency_demand;
pub mod resident_target;
pub mod source_retirement;
use mirror_state::{CurrentMirror, MirrorRegion};
pub use residency_demand::MirrorResidencyDemandBinding;
pub use resident_target::ResidentMirrorTarget;

pub use edition_copy_lane::PartitionEditionCopyLane;
pub use edition_writer::{
    PartitionEditionWriter, ResidentPartitionEditionWriter, ResidentPartitionEntry,
};
pub use partition_edition::{PartitionEdition, PartitionMap, ResidentPartitionEdition};

use crate::identity::data::PartitionId;
use crate::performance::data::runtime_complexity_counters::RuntimeComplexityCounters;
use crate::storage::overlay::PartitionState;
use crate::storage::residency::RelationalRegionRetention;
#[cfg(test)]
use crate::storage::residency::{Full, RelationalResidencyLease};
use crate::storage::substrate::owned_state::RuntimeOwnedState;
use std::sync::Mutex;

/// The runtime's record substrate, owned behind its own lock.
///
/// Readers never retain the lock and never copy the substrate: `acquire`
/// returns a structurally shared edition after a single atomic increment and
/// releases the lock immediately, so no read path can deadlock against a
/// settling executor and no lock is ever held across derived work.
///
/// Writers take the lock exclusively, retain persistent map roots in O(1), and
/// copy changed map paths plus any observed partition they actually mutate.
#[derive(Debug, Default)]
pub struct StorageSubsystem {
    partitions: RuntimeOwnedState<CurrentMirror>,
}

impl StorageSubsystem {
    pub fn install_cells(&self, cells: PartitionMap) {
        *self.partitions.write() = CurrentMirror::from_cells(cells);
    }
    /// One structurally shared edition of the substrate. O(1): one atomic
    /// increment, no allocation, no retained lock.
    pub fn acquire(&self) -> PartitionEdition {
        self.partitions.read().edition.clone()
    }

    /// Exclusive authority to install the next edition, charged to `lane`.
    pub fn edit<'subsystem>(
        &'subsystem self,
        copy_counters: &'subsystem Mutex<RuntimeComplexityCounters>,
        lane: PartitionEditionCopyLane,
    ) -> PartitionEditionWriter<'subsystem> {
        PartitionEditionWriter::new(self.partitions.write(), copy_counters, lane)
    }

    /// Demand outside the write lock, then seal the current edition. Publication
    /// installs resident entries; only an exclusive recovery can install cold ones.
    pub fn demand_resident_edit<'subsystem>(
        &'subsystem self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        copy_counters: &'subsystem Mutex<RuntimeComplexityCounters>,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Result<
        ResidentPartitionEditionWriter<'subsystem>,
        crate::storage::residency::RelationalResidencyDemandFailure,
    > {
        loop {
            self.acquire().demand_all(execution, symbols)?;
            match self
                .edit(copy_counters, PartitionEditionCopyLane::Ordinary)
                .accept_resident(symbols)
            {
                Ok(writer) => return Ok(writer),
                Err(writer) => drop(writer),
            }
        }
    }

    pub fn partition_ids(&self) -> Vec<PartitionId> {
        use crate::storage::overlay::CurrentPartitionAccess;
        self.partitions.read().edition.partition_ids()
    }

    #[cfg(test)]
    pub fn partition(&self, partition_id: PartitionId) -> Option<RelationalResidencyLease<Full>> {
        self.partitions
            .read()
            .edition
            .shared_partition(partition_id)
            .expect("test fixture admits a resident partition")
    }

    pub fn install_owned(
        &self,
        copy_counters: &Mutex<RuntimeComplexityCounters>,
        lane: PartitionEditionCopyLane,
        partitions: impl IntoIterator<Item = (PartitionId, PartitionState)>,
    ) {
        let installed: PartitionMap = partitions
            .into_iter()
            .map(|(partition_id, partition)| {
                (partition_id, RelationalRegionRetention::resident(partition))
            })
            .collect();
        self.edit(copy_counters, lane).install(installed);
    }
}

impl StorageSubsystem {
    /// Share immutable cells; runtime pin counters detach at the first write.
    /// Cold cells remain cold; current edits carry Head, history edits carry Full.
    pub fn detached(&self) -> Self {
        let partitions = self
            .acquire()
            .cells()
            .map(|(id, region)| (id, region.fork()))
            .collect();
        Self {
            partitions: RuntimeOwnedState::new(CurrentMirror::from_cells(partitions)),
        }
    }
}

pub use edition_writer::TryPartitionMutationDenied;
