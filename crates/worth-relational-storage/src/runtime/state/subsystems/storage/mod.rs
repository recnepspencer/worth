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
    /// This runtime's bounded working set, fixed by the first configured
    /// budget it sees; every cold-backed region it readmits joins it.
    working_set: std::sync::OnceLock<
        Option<std::sync::Arc<crate::storage::residency::working_set::WorkingSetAccountant>>,
    >,
}

impl StorageSubsystem {
    /// The working-set authority for udget, created once per runtime.
    pub fn working_set(
        &self,
        budget: Option<crate::storage::residency::working_set::ResidencyBudget>,
    ) -> Option<std::sync::Arc<crate::storage::residency::working_set::WorkingSetAccountant>> {
        self.working_set
            .get_or_init(|| {
                budget.map(crate::storage::residency::working_set::WorkingSetAccountant::new)
            })
            .clone()
    }

    /// Point-in-time residency pressure; None for an unbounded runtime.
    pub fn residency_pressure(
        &self,
    ) -> Option<crate::storage::residency::working_set::ResidencyPressureSnapshot> {
        self.working_set
            .get()
            .and_then(Option::as_ref)
            .map(|accountant| accountant.pressure())
    }

    pub fn install_cells(&self, cells: PartitionMap) {
        *self.partitions.write() = CurrentMirror::from_cells(cells);
    }
    /// One structurally shared edition of the substrate with its own empty
    /// operation residency. O(1): one atomic increment, no retained lock.
    pub fn acquire(&self) -> PartitionEdition {
        self.partitions.read().edition.acquired()
    }

    /// Diagnostic edition with its own empty residency that counts no
    /// acquisition: it observes what the cells already serve, with no I/O.
    pub fn observe_edition(&self) -> PartitionEdition {
        self.partitions.read().edition.acquired()
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
            // The writer admits targets from the mirror's own custody.
            let mirror = self.partitions.read().edition.clone();
            mirror.demand_all(execution, symbols)?;
            drop(mirror);
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
                let lease =
                    crate::storage::residency::RelationalRegionResidency::resident_uncommitted(
                        partition,
                    );
                (partition_id, RelationalRegionRetention::mirror_owned(lease))
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
            working_set: self.working_set.clone(),
        }
    }
}

pub use edition_writer::TryPartitionMutationDenied;
