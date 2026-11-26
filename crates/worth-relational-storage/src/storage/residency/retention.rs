//! A root retains only its cell's durable member custody. Every lease a read
//! admits lives in the demanding operation's [`OperationResidency`], so keeping
//! a root open never pins a resident image past the operation that read it
//! (Store M4 D1). Only the runtime's current mirror keeps per-entry owner
//! slots: they hold its edit targets and drop with the entry it replaces.
//! Accessors never perform storage I/O; cold cells must be demanded first,
//! into the same custody that later lends their views.
use std::sync::Arc;

use super::operation_residency::{OperationResidency, ScopedRegion};
use super::{Full, Head, RelationalRegionResidency, RelationalResidencyLease};
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::overlay::PartitionState;

type Failure = super::RelationalResidencyDemandFailure;

mod accepted;

#[derive(Debug)]
pub struct RelationalRegionRetention {
    cell: Arc<RelationalRegionResidency>,
    authority: Option<Arc<super::RelationalRegionKindAuthority>>,
    owner: ScopedRegion,
    clear_pins_on_write: bool,
}

/// Where admitted leases live: one operation's scope, or the mirror's entry.
#[derive(Clone, Copy)]
pub enum ResidencyCustody<'s> {
    Operation(&'s OperationResidency),
    Mirror(MirrorOwnerSlot<'s>),
}

/// The current mirror's own per-entry slot; only the storage owner names it.
#[derive(Clone, Copy)]
pub struct MirrorOwnerSlot<'s>(&'s ScopedRegion);

impl<'s> From<&'s OperationResidency> for ResidencyCustody<'s> {
    fn from(scope: &'s OperationResidency) -> Self {
        Self::Operation(scope)
    }
}

impl<'s> ResidencyCustody<'s> {
    fn slot(self, cell: &Arc<RelationalRegionResidency>) -> &'s ScopedRegion {
        match self {
            Self::Operation(scope) => scope.slot(cell),
            Self::Mirror(MirrorOwnerSlot(slot)) => slot,
        }
    }

    fn existing(self, cell: &Arc<RelationalRegionResidency>) -> Option<&'s ScopedRegion> {
        match self {
            Self::Operation(scope) => scope.existing(cell),
            Self::Mirror(MirrorOwnerSlot(slot)) => Some(slot),
        }
    }
}

impl Clone for RelationalRegionRetention {
    fn clone(&self) -> Self {
        let owner = ScopedRegion::default();
        if let Some(lease) = self.owner.full.get() {
            let _ = owner.full.set(lease.clone());
        }
        if let Some(lease) = self.owner.head.get() {
            let _ = owner.head.set(lease.clone());
        }
        Self {
            cell: self.cell.clone(),
            authority: self.authority.clone(),
            owner,
            clear_pins_on_write: self.clear_pins_on_write,
        }
    }
}

impl RelationalRegionRetention {
    pub fn new(
        cell: Arc<RelationalRegionResidency>,
        authority: Option<Arc<super::RelationalRegionKindAuthority>>,
    ) -> Self {
        Self {
            cell,
            authority,
            owner: ScopedRegion::default(),
            clear_pins_on_write: false,
        }
    }

    pub fn resident(partition: PartitionState) -> Self {
        Self::from_lease(RelationalRegionResidency::resident_uncommitted(partition))
    }

    /// The cell keeps serving its published image; the publishing lease is
    /// released here, never retained for the root's lifetime.
    pub fn from_lease(lease: RelationalResidencyLease<Full>) -> Self {
        Self::new(lease.cell().clone(), None)
    }

    pub fn from_head_lease(lease: RelationalResidencyLease<Head>) -> Self {
        Self::new(lease.cell().clone(), None)
    }

    /// A mirror entry that keeps its edit target's lease until replaced.
    pub(crate) fn mirror_owned(lease: RelationalResidencyLease<Full>) -> Self {
        let retention = Self::new(lease.cell().clone(), None);
        let _ = retention.owner.full.set(lease);
        retention
    }

    pub(crate) fn mirror_owned_head(lease: RelationalResidencyLease<Head>) -> Self {
        let retention = Self::new(lease.cell().clone(), None);
        let _ = retention.owner.head.set(lease);
        retention
    }

    pub(crate) fn mirror_custody(&self) -> ResidencyCustody<'_> {
        ResidencyCustody::Mirror(MirrorOwnerSlot(&self.owner))
    }

    pub fn cell(&self) -> &Arc<RelationalRegionResidency> {
        &self.cell
    }

    pub fn kind_authority(&self) -> Option<Arc<super::RelationalRegionKindAuthority>> {
        self.authority.clone()
    }

    pub fn reconstruct_head(
        &self,
        lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::ReconstructedRegionHead, Failure> {
        self.cell
            .reconstruct_head(lane, execution, self.authority.as_deref())
    }

    fn unavailable(&self) -> RelationalStorageUnavailable {
        RelationalStorageUnavailable {
            subject: RelationalStorageSubject::Region(self.cell.partition_id),
            cause: RelationalStorageUnavailableCause::SourceUnavailable,
        }
    }

    /// Owned Full borrow that no root or scope retains; dropping it releases
    /// the counted borrow and lets a bound cell be evicted.
    pub fn borrow_full(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Full>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        self.cell.check_damage()?;
        self.cell
            .demand_full(execution, limit, self.authority.as_deref())
    }

    /// Owned Head borrow with the same non-retaining contract.
    pub fn borrow_head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Head>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        self.cell.check_damage()?;
        self.cell
            .demand_head(execution, limit, self.authority.as_deref())
    }

    /// Demand Full residency into `custody`, which then lends its views.
    pub fn demand_full<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Full>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        self.cell.check_damage()?;
        let slot = custody.into().slot(&self.cell);
        if let Some(lease) = slot.full.get() {
            return Ok(lease.clone());
        }
        let lease = self
            .cell
            .demand_full(execution, limit, self.authority.as_deref())?;
        let _ = slot.full.set(lease.clone());
        Ok(lease)
    }

    pub fn demand_head_cached<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<(), Failure> {
        self.demand_head(custody, execution, limit).map(drop)
    }

    pub fn demand_head<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        limit: u64,
    ) -> Result<RelationalResidencyLease<Head>, Failure> {
        crate::execution::check_request(execution).map_err(Failure::Request)?;
        self.cell.check_damage()?;
        let slot = custody.into().slot(&self.cell);
        if let Some(lease) = slot.full.get() {
            return Ok(lease.as_head());
        }
        if let Some(lease) = slot.head.get() {
            return Ok(lease.clone());
        }
        let lease = self
            .cell
            .demand_head(execution, limit, self.authority.as_deref())?;
        let _ = slot.head.set(lease.clone());
        Ok(lease)
    }

    pub fn capture_region<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Result<Option<&'s super::ResidentRegion>, RelationalStorageUnavailable> {
        self.cell.check_damage()?;
        let slot = custody.into().slot(&self.cell);
        if let Some(lease) = slot.full.get() {
            return Ok(Some(lease.resident_region()));
        }
        if slot.head.get().is_none()
            && let Some(lease) = self.cell.lease_resident_head()?
        {
            let _ = slot.head.set(lease);
        }
        Ok(slot.head.get().map(|lease| lease.resident_region()))
    }

    /// Diagnostic census: whether the cell serves Head / Full with `symbols`
    /// admitted. Grants no custody, performs no I/O, and needs no operation.
    pub fn residency_census(
        &self,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> (bool, bool) {
        use super::working_set::ResidencyLevelKind as Level;
        use super::RelationalRegionSymbolAdmissionIdentity as Identity;
        let level = self.cell.resident_level();
        let head = level.is_some() && symbols.region_symbols_admitted(Identity::head(&self.cell));
        let full = level == Some(Level::Full)
            && symbols.region_symbols_admitted(Identity::full(&self.cell));
        (head, full)
    }

    /// No other root, edition or live borrow observes this cell.
    pub fn is_uniquely_owned(&self) -> bool {
        Arc::strong_count(&self.cell)
            == 1 + usize::from(self.owner.full.get().is_some())
                + usize::from(self.owner.head.get().is_some())
    }

    pub fn fork(&self) -> Self {
        let mut fork = Self::new(self.cell.clone(), self.authority.clone());
        fork.clear_pins_on_write = true;
        fork
    }

    pub fn clears_pins_on_write(&self) -> bool {
        self.clear_pins_on_write
    }
}
