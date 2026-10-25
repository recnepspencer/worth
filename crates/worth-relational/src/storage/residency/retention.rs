//! A root or edition retains its cell and any full residency it has admitted.
//! Accessors never perform storage I/O. A cold cell must be demanded explicitly
//! by the operation before a full view can be borrowed.
use std::sync::{Arc, OnceLock};

use super::{Full, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

#[derive(Debug)]
pub(crate) struct RelationalRegionRetention {
    cell: Arc<RelationalRegionResidency>,
    full: OnceLock<RelationalResidencyLease<Full>>,
}

impl RelationalRegionRetention {
    pub(crate) fn new(cell: Arc<RelationalRegionResidency>) -> Self {
        Self {
            cell,
            full: OnceLock::new(),
        }
    }

    pub(crate) fn resident(partition: PartitionState) -> Self {
        let lease = RelationalRegionResidency::resident_uncommitted(partition);
        Self::from_lease(lease)
    }

    pub(crate) fn from_lease(lease: RelationalResidencyLease<Full>) -> Self {
        let cell = lease.cell().clone();
        Self {
            cell,
            full: OnceLock::from(lease),
        }
    }

    pub(crate) fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }

    pub(crate) fn demand_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<(), super::RelationalResidencyDemandFailure> {
        if self.full.get().is_none() {
            let lease = self.cell.demand_full(execution, limit)?;
            let _ = self.full.set(lease);
        }
        Ok(())
    }

    pub(crate) fn lease(&self) -> &RelationalResidencyLease<Full> {
        self.full.get_or_init(|| {
            self.cell
                .lease_resident_full()
                .expect("resident lease admission fits the process")
                .expect("operation must demand a cold region before reading it")
        })
    }

    pub(crate) fn full_view(&self) -> super::RelationalRegionFullView<'_> {
        self.lease().full_view()
    }
    pub(crate) fn residency_allocation_bytes(&self) -> u64 {
        self.lease().residency_allocation_bytes()
    }

    pub(crate) fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell) == 1 + usize::from(self.full.get().is_some())
    }

    pub(crate) fn into_partition(self) -> PartitionState {
        drop(self.full.into_inner());
        match Arc::try_unwrap(self.cell) {
            Ok(cell) => cell.into_partition(),
            Err(cell) => cell
                .lease_resident_full()
                .expect("resident lease admission fits the process")
                .expect("publication requires resident substrate")
                .full_view()
                .partition()
                .clone(),
        }
    }

    pub(crate) fn make_mut(&mut self) -> &mut PartitionState {
        // Drop our read lease before asking for exclusive cell ownership.
        drop(self.full.take());
        if Arc::get_mut(&mut self.cell).is_none() {
            let lease = self
                .cell
                .lease_resident_full()
                .expect("resident lease admission fits the process")
                .expect("writer must demand its partition before editing it");
            let copied = lease.full_view().partition().clone();
            drop(lease);
            let lease = RelationalRegionResidency::resident_uncommitted(copied);
            self.cell = lease.cell().clone();
            drop(lease);
        }
        Arc::get_mut(&mut self.cell)
            .expect("writer owns its cell")
            .partition_mut()
    }
}

impl Clone for RelationalRegionRetention {
    fn clone(&self) -> Self {
        let full = OnceLock::new();
        if let Some(lease) = self.full.get() {
            let _ = full.set(lease.clone());
        }
        Self {
            cell: self.cell.clone(),
            full,
        }
    }
}

impl AsRef<PartitionState> for RelationalRegionRetention {
    fn as_ref(&self) -> &PartitionState {
        self.lease().full_view().partition()
    }
}

impl std::ops::Deref for RelationalRegionRetention {
    type Target = PartitionState;
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
