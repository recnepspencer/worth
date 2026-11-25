//! A root or edition retains its cell and any residency it has admitted
//! through its caching doors. `borrow_head`/`borrow_full` are operation-scoped
//! instead: they never retain, so an open root does not pin bound cells.
//! Accessors never perform storage I/O; cold cells must be demanded first.
use std::sync::{Arc, OnceLock};

use super::{Full, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::overlay::PartitionState;

#[derive(Debug)]
pub struct RelationalRegionRetention {
    cell: Arc<RelationalRegionResidency>,
    authority: Option<Arc<super::RelationalRegionKindAuthority>>,
    full: OnceLock<RelationalResidencyLease<Full>>,
    head: OnceLock<RelationalResidencyLease<super::Head>>,
    clear_pins_on_write: bool,
}

impl RelationalRegionRetention {
    pub fn retire_native_leases(&self) {
        if let Some(lease) = self.full.get() {
            lease.retire_native();
        }
        if let Some(lease) = self.head.get() {
            lease.retire_native();
        }
    }

    pub fn new(
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

    pub fn resident(partition: PartitionState) -> Self {
        let lease = RelationalRegionResidency::resident_uncommitted(partition);
        Self::from_lease(lease)
    }

    pub fn from_lease(lease: RelationalResidencyLease<Full>) -> Self {
        let cell = lease.cell().clone();
        Self {
            cell,
            authority: None,
            full: OnceLock::from(lease),
            head: OnceLock::new(),
            clear_pins_on_write: false,
        }
    }

    pub fn from_head_lease(lease: RelationalResidencyLease<super::Head>) -> Self {
        Self {
            cell: lease.cell().clone(),
            authority: None,
            full: OnceLock::new(),
            head: OnceLock::from(lease),
            clear_pins_on_write: false,
        }
    }

    pub fn capture_region(
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

    pub fn reconstruct_head(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::ReconstructedRegionHead, super::RelationalResidencyDemandFailure> {
        self.cell
            .reconstruct_head(lane, execution, self.authority.as_deref())
    }

    pub fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }

    pub fn demand_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Full>, super::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        self.cell.check_damage()?;
        if let Some(lease) = self.full.get() {
            return Ok(lease.clone());
        }
        let lease = self
            .cell
            .demand_full(execution, limit, self.authority.as_deref())?;
        let _ = self.full.set(lease.clone());
        Ok(lease)
    }

    /// Operation-scoped Full borrow: reuses custody this root already holds,
    /// otherwise demands and returns a lease that this root never retains.
    /// Dropping the lease releases the borrow, so a long-lived root does not
    /// pin the image and a bound cell becomes evictable again.
    pub fn borrow_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Full>, super::RelationalResidencyDemandFailure> {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        self.cell.check_damage()?;
        match self.full.get() {
            Some(lease) => Ok(lease.clone()),
            None => self
                .cell
                .demand_full(execution, limit, self.authority.as_deref()),
        }
    }

    /// Operation-scoped Head borrow with the same non-retaining contract.
    pub fn borrow_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<super::Head>, super::RelationalResidencyDemandFailure>
    {
        crate::execution::check_request(execution)
            .map_err(super::RelationalResidencyDemandFailure::Request)?;
        self.cell.check_damage()?;
        match self.accepted_head_lease() {
            Some(lease) => Ok(lease),
            None => self
                .cell
                .demand_head(execution, limit, self.authority.as_deref()),
        }
    }

    /// Lend only a previously admitted full view. A probe records `None` as a
    /// miss and discards its provisional result before any demand can run.
    pub fn admitted_partition(&self) -> Option<&PartitionState> {
        // A probe turns every unavailable state into a discovered demand. The
        // round discards this provisional absence before the typed demand runs.
        self.try_admitted_partition().ok().flatten()
    }

    pub fn try_admitted_partition(
        &self,
    ) -> Result<Option<&PartitionState>, super::super::data::RelationalStorageUnavailable> {
        self.cell.check_damage()?;
        if self.full.get().is_none()
            && let Some(lease) = self.cell.lease_resident_full()?
        {
            let _ = self.full.set(lease);
        }
        Ok(self.full.get().map(|lease| lease.full_view().partition()))
    }

    /// Borrow custody already accepted by this immutable root. Fresh operations
    /// must demand first; later transport damage cannot revoke this lease.
    pub fn accepted_head_view(&self) -> Option<super::RelationalRegionHeadView<'_>> {
        self.full
            .get()
            .map(|lease| lease.head_view())
            .or_else(|| self.head.get().map(|lease| lease.head_view()))
    }

    pub fn admitted_head_view(&self) -> Option<super::RelationalRegionHeadView<'_>> {
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

    pub fn head_symbols_admitted(
        &self,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
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

    pub fn demand_head_cached(
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

    pub fn pin_lease(
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

    /// The root retains this exact Full lease; accepted read capabilities can
    /// keep using its immutable bytes after a later fresh demand finds damage.
    pub fn accepted_full_partition(&self) -> Option<&PartitionState> {
        self.full.get().map(|lease| lease.full_view().partition())
    }

    /// Pure read-only peek of an already-accepted Full lease whose symbols are
    /// admitted in `symbols`. Never queries the shared cell or installs a lease.
    pub fn peek_full_partition_with_symbols(
        &self,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Option<&PartitionState> {
        let lease = self.full.get()?;
        symbols
            .region_symbols_admitted(lease.symbol_admission_identity())
            .then(|| lease.full_view().partition())
    }

    /// Pure read-only peek of an already-accepted Head (or Full) lease whose
    /// head symbols are admitted in `symbols`. Never queries the shared cell or
    /// installs a lease.
    pub fn peek_head_view_with_symbols(
        &self,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Option<super::RelationalRegionHeadView<'_>> {
        let view = self.accepted_head_view()?;
        self.head_symbols_admitted(symbols).then_some(view)
    }

    /// Clone custody already accepted by this immutable root, without a fresh
    /// admission decision. A cold cell alone supplies no witness here.
    pub fn accepted_head_lease(&self) -> Option<RelationalResidencyLease<super::Head>> {
        self.head
            .get()
            .cloned()
            .or_else(|| self.full.get().map(|lease| lease.as_head()))
    }

    pub fn pin_head_lease(
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

    pub fn kind_authority(&self) -> Option<Arc<super::RelationalRegionKindAuthority>> {
        self.authority.clone()
    }
    pub fn admitted_full_lease(&self) -> Option<RelationalResidencyLease<Full>> {
        self.full.get().cloned()
    }
    pub fn demand_head(
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

    pub fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell)
            == 1 + usize::from(self.full.get().is_some()) + usize::from(self.head.get().is_some())
    }

    pub fn fork(&self) -> Self {
        let mut fork = self.clone();
        fork.clear_pins_on_write = true;
        fork
    }

    pub fn clears_pins_on_write(&self) -> bool {
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
