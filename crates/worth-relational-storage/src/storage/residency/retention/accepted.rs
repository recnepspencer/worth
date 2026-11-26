//! Accepted custody: what this operation already admitted, or an image the
//! cell already serves, borrowed with no I/O (Store M4 D1). Keeping a root
//! open retains nothing, so cell residency, not root retention, decides
//! whether a no-I/O read can be served.
use super::{RelationalRegionRetention, ResidencyCustody};
use crate::storage::data::RelationalStorageUnavailable;
use crate::storage::overlay::PartitionState;
use crate::storage::residency::operation_residency::ScopedRegion;
use crate::storage::residency::working_set::ResidencyLevelKind;
use crate::storage::residency::{Full, Head, RelationalResidencyLease};

type SymbolTable = crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;

impl RelationalRegionRetention {
    /// This custody's slot once it holds a lease, admitting the cell's served
    /// image into it first: an operation's read, or the mirror's edit target.
    /// `want` is the level the caller reads; a held Full covers Head.
    fn served<'s>(
        &'s self,
        custody: ResidencyCustody<'s>,
        want: ResidencyLevelKind,
    ) -> Option<&'s ScopedRegion> {
        let any = |slot: &&ScopedRegion| slot.full.get().is_some() || slot.head.get().is_some();
        let held = custody.existing(&self.cell).filter(any);
        if held.is_some_and(|slot| slot.full.get().is_some() || want == ResidencyLevelKind::Head) {
            return held;
        }
        // A cold probe allocates no slot and pins no cell.
        let Some((region, level)) = self.cell.lease_served(self.authority.as_deref()) else {
            return held;
        };
        let slot = custody.slot(&self.cell);
        if level == ResidencyLevelKind::Full {
            let lease = RelationalResidencyLease::<Full>::accepted(self.cell.clone(), region);
            let _ = slot.full.set(lease);
        } else {
            let lease = RelationalResidencyLease::<Head>::accepted(self.cell.clone(), region);
            let _ = slot.head.set(lease);
        }
        Some(slot).filter(any)
    }

    /// Lend only an admitted or served full view. A probe records `None` as
    /// a miss and discards its provisional result before any demand can run.
    pub fn admitted_partition<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<&'s PartitionState> {
        self.try_admitted_partition(custody).ok().flatten()
    }

    pub fn try_admitted_partition<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Result<Option<&'s PartitionState>, RelationalStorageUnavailable> {
        self.cell.check_damage()?;
        Ok(self
            .served(custody.into(), ResidencyLevelKind::Full)
            .and_then(|slot| slot.full.get())
            .map(|lease| lease.full_view().partition()))
    }

    /// Custody already accepted; later transport damage cannot revoke it.
    pub fn accepted_head_view<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<super::super::RelationalRegionHeadView<'s>> {
        let slot = self.served(custody.into(), ResidencyLevelKind::Head)?;
        slot.full
            .get()
            .map(|lease| lease.head_view())
            .or_else(|| slot.head.get().map(|lease| lease.head_view()))
    }

    /// Head view only if `custody` already holds an accepted lease for this
    /// cell; never pins an unborrowed warm cell into `custody`.
    pub fn held_head_view<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<super::super::RelationalRegionHeadView<'s>> {
        let slot = custody.into().existing(&self.cell)?;
        slot.full
            .get()
            .map(|lease| lease.head_view())
            .or_else(|| slot.head.get().map(|lease| lease.head_view()))
    }

    pub fn admitted_head_view<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<super::super::RelationalRegionHeadView<'s>> {
        self.cell.check_damage().ok()?;
        self.accepted_head_view(custody)
    }

    pub fn head_symbols_admitted<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        symbols: &SymbolTable,
    ) -> bool {
        let Some(slot) = self.served(custody.into(), ResidencyLevelKind::Head) else {
            return false;
        };
        let identity = slot
            .full
            .get()
            .map(|lease| lease.head_symbol_admission_identity())
            .or_else(|| {
                slot.head
                    .get()
                    .map(|lease| lease.head_symbol_admission_identity())
            });
        identity.is_some_and(|identity| symbols.region_symbols_admitted(identity))
    }

    /// Accepted or served Full custody; never reads storage.
    pub fn pin_lease<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Result<RelationalResidencyLease<Full>, RelationalStorageUnavailable> {
        self.cell.check_damage()?;
        self.admitted_full_lease(custody)
            .ok_or_else(|| self.unavailable())
    }

    pub fn pin_head_lease<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Result<RelationalResidencyLease<Head>, RelationalStorageUnavailable> {
        self.cell.check_damage()?;
        self.accepted_head_lease(custody)
            .ok_or_else(|| self.unavailable())
    }

    /// The exact accepted Full lease; never reads storage.
    pub fn accepted_full_partition<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<&'s PartitionState> {
        let lease = self
            .served(custody.into(), ResidencyLevelKind::Full)?
            .full
            .get()?;
        Some(lease.full_view().partition())
    }

    /// Pure peek of accepted Full custody whose symbols `symbols` admitted.
    pub fn peek_full_partition_with_symbols<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        symbols: &SymbolTable,
    ) -> Option<&'s PartitionState> {
        let lease = self
            .served(custody.into(), ResidencyLevelKind::Full)?
            .full
            .get()?;
        symbols
            .region_symbols_admitted(lease.symbol_admission_identity())
            .then(|| lease.full_view().partition())
    }

    /// Pure peek of accepted Head (or Full) custody with admitted head symbols.
    pub fn peek_head_view_with_symbols<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
        symbols: &SymbolTable,
    ) -> Option<super::super::RelationalRegionHeadView<'s>> {
        let custody = custody.into();
        let view = self.accepted_head_view(custody)?;
        self.head_symbols_admitted(custody, symbols).then_some(view)
    }

    /// Clone accepted custody without a fresh load.
    pub fn accepted_head_lease<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<RelationalResidencyLease<Head>> {
        let slot = self.served(custody.into(), ResidencyLevelKind::Head)?;
        slot.head
            .get()
            .cloned()
            .or_else(|| slot.full.get().map(|lease| lease.as_head()))
    }

    pub fn admitted_full_lease<'s>(
        &'s self,
        custody: impl Into<ResidencyCustody<'s>>,
    ) -> Option<RelationalResidencyLease<Full>> {
        self.served(custody.into(), ResidencyLevelKind::Full)?
            .full
            .get()
            .cloned()
    }
}
