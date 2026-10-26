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
    head: OnceLock<RelationalResidencyLease<super::Head>>,
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
            head: OnceLock::new(),
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
            head: OnceLock::new(),
            clear_pins_on_write: false,
        }
    }

    pub(crate) fn from_head_lease(lease: RelationalResidencyLease<super::Head>) -> Self {
        Self {
            cell: lease.cell().clone(),
            authority: None,
            full: OnceLock::new(),
            head: OnceLock::from(lease),
            clear_pins_on_write: false,
        }
    }

    pub(super) fn capture_region(
        &self,
    ) -> Result<Option<&super::ResidentRegion>, crate::storage::data::RelationalStorageUnavailable>
    {
        self.cell.check_damage()?;
        if let Some(lease) = self.full.get() {
            return Ok(Some(lease.resident_region()));
        }
        if self.head.get().is_none()
            && let Some(lease) = self.cell.lease_resident_head()?
        {
            let _ = self.head.set(lease);
        }
        Ok(self.head.get().map(|lease| lease.resident_region()))
    }

    pub(crate) fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }

    pub(crate) fn demand_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<(), super::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        self.cell.check_damage()?;
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

    /// Lend only a previously admitted full view. A probe records `None` as a
    /// miss and discards its provisional result before any demand can run.
    pub(crate) fn admitted_partition(&self) -> Option<&PartitionState> {
        self.cell.check_damage().ok()?;
        if self.full.get().is_none() {
            let lease = self.cell.lease_resident_full().ok().flatten()?;
            let _ = self.full.set(lease);
        }
        self.full.get().map(|lease| lease.full_view().partition())
    }

    pub(crate) fn admitted_head_view(&self) -> Option<super::RelationalRegionHeadView<'_>> {
        self.cell.check_damage().ok()?;
        if let Some(full) = self.full.get() {
            return Some(full.head_view());
        }
        if self.head.get().is_none() {
            let lease = self.cell.lease_resident_head().ok().flatten()?;
            let _ = self.head.set(lease);
        }
        self.head.get().map(|lease| lease.head_view())
    }

    pub(crate) fn head_symbols_admitted(
        &self,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> bool {
        let identity = self
            .full
            .get()
            .map(|lease| lease.head_symbol_admission_identity())
            .or_else(|| {
                self.head
                    .get()
                    .map(|lease| lease.head_symbol_admission_identity())
            });
        identity.is_some_and(|identity| symbols.region_symbols_admitted(identity))
    }

    pub(crate) fn demand_head_cached(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<(), super::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        self.cell.check_damage()?;
        if self.full.get().is_none() && self.head.get().is_none() {
            let lease = self
                .cell
                .demand_head(execution, limit, self.authority.as_deref())?;
            let _ = self.head.set(lease);
        }
        Ok(())
    }

    pub(crate) fn pin_lease(
        &self,
    ) -> Result<RelationalResidencyLease<Full>, crate::storage::data::RelationalStorageUnavailable>
    {
        self.cell.check_damage()?;
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

    pub(crate) fn pin_head_lease(
        &self,
    ) -> Result<
        RelationalResidencyLease<super::Head>,
        crate::storage::data::RelationalStorageUnavailable,
    > {
        self.cell.check_damage()?;
        self.head
            .get()
            .cloned()
            .or_else(|| self.full.get().map(|lease| lease.as_head()))
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
    pub(crate) fn admitted_full_lease(&self) -> Option<RelationalResidencyLease<Full>> {
        self.full.get().cloned()
    }
    pub(crate) fn demand_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<super::RelationalResidencyLease<super::Head>, super::RelationalResidencyDemandFailure>
    {
        self.demand_head_cached(execution, limit)?;
        if let Some(head) = self.head.get() {
            return Ok(head.clone());
        }
        self.cell
            .demand_head(execution, limit, self.authority.as_deref())
    }

    pub(crate) fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell)
            == 1 + usize::from(self.full.get().is_some()) + usize::from(self.head.get().is_some())
    }

    pub(crate) fn fork(&self) -> Self {
        let mut fork = self.clone();
        fork.clear_pins_on_write = true;
        fork
    }

    pub(crate) fn clears_pins_on_write(&self) -> bool {
        self.clear_pins_on_write
    }
}

impl Clone for RelationalRegionRetention {
    fn clone(&self) -> Self {
        let full = OnceLock::new();
        if let Some(lease) = self.full.get() {
            let _ = full.set(lease.clone());
        }
        let head = OnceLock::new();
        if let Some(lease) = self.head.get() {
            let _ = head.set(lease.clone());
        }
        Self {
            cell: self.cell.clone(),
            authority: self.authority.clone(),
            full,
            head,
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
