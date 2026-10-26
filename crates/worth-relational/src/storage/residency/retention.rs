//! A root or edition retains its cell and any full residency it has admitted.
//! Accessors never perform storage I/O. A cold cell must be demanded explicitly
//! by the operation before a full view can be borrowed.
use std::sync::{Arc, OnceLock};

use super::{Full, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

#[derive(Debug)]
pub(crate) struct RelationalRegionRetention {
    cell: Arc<RelationalRegionResidency>,
    authority: Option<Arc<super::RelationalRegionKindAuthority>>,
    full: OnceLock<RelationalResidencyLease<Full>>,
    clear_pins_on_write: bool,
}

impl RelationalRegionRetention {
    pub(crate) fn new(
        cell: Arc<RelationalRegionResidency>,
        authority: Option<Arc<super::RelationalRegionKindAuthority>>,
    ) -> Self {
        Self {
            cell,
            authority,
            full: OnceLock::new(),
            clear_pins_on_write: false,
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
            authority: None,
            full: OnceLock::from(lease),
            clear_pins_on_write: false,
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
            let lease = self
                .cell
                .demand_full(execution, limit, self.authority.as_deref())?;
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

    pub(crate) fn pin_lease(
        &self,
    ) -> Result<RelationalResidencyLease<Full>, crate::storage::data::RelationalStorageUnavailable>
    {
        if let Some(lease) = self.full.get() {
            return Ok(lease.clone());
        }
        self.cell
            .lease_resident_full()?
            .ok_or(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Region(
                    self.cell.partition_id,
                ),
                cause: crate::storage::data::RelationalStorageUnavailableCause::SourceUnavailable,
            })
    }

    pub(crate) fn kind_authority(&self) -> Option<Arc<super::RelationalRegionKindAuthority>> {
        self.authority.clone()
    }
    pub(crate) fn demand_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<super::RelationalResidencyLease<super::Head>, super::RelationalResidencyDemandFailure>
    {
        self.cell
            .demand_head(execution, limit, self.authority.as_deref())
    }

    pub(crate) fn full_view(&self) -> super::RelationalRegionFullView<'_> {
        self.lease().full_view()
    }
    pub(crate) fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell) == 1 + usize::from(self.full.get().is_some())
    }

    pub(crate) fn into_partition(self) -> PartitionState {
        let full = self.full.into_inner();
        if self.cell.has_cold_backing() {
            return full
                .expect("cold publication requires its admitted root view")
                .full_view()
                .partition()
                .clone();
        }
        drop(full);
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

    /// A cold retention cannot produce mutable access. Its operation must first
    /// acquire a Full lease through the explicit, fallible demand boundary.
    pub(crate) fn make_mut(&mut self) -> Option<&mut PartitionState> {
        let lease = self
            .full
            .take()
            .or_else(|| self.cell.lease_resident_full().ok().flatten())?;
        let retained = lease.retained_region();
        drop(lease);
        if self.cell.has_cold_backing() || Arc::get_mut(&mut self.cell).is_none() {
            let lease = RelationalRegionResidency::resident_uncommitted(retained.partition.clone());
            self.cell = lease.cell().clone();
            drop(lease);
            self.authority = None;
        }
        drop(retained);
        let partition = Arc::get_mut(&mut self.cell)?.partition_mut()?;
        if self.clear_pins_on_write {
            partition.clear_runtime_pin_counters();
            self.clear_pins_on_write = false;
        }
        Some(partition)
    }

    pub(crate) fn fork(&self) -> Self {
        let mut fork = self.clone();
        fork.clear_pins_on_write = true;
        fork
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
            authority: self.authority.clone(),
            full,
            clear_pins_on_write: self.clear_pins_on_write,
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
