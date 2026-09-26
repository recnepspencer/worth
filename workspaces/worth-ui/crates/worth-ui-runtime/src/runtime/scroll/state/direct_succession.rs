//! Direct input is a bounded pending Scroll result until its exact frame is
//! accepted. Copies carried by mounted publication are evidence, not writers.
use super::{UiScrollOwnerRecord, UiScrollRuntimeState};
use crate::runtime::scroll::{UiScrollDeltaCause, UiScrollOwnerIdentity, UiScrollRouteReceipt};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiPreparedScrollDirectSuccession {
    owner: UiScrollOwnerIdentity,
    occurrence: UiMountedInstanceIdentity,
    geometry_owner: Option<UiMountedInstanceIdentity>,
    pub(super) record: UiScrollOwnerRecord,
    cause: UiScrollDeltaCause,
    revision: u64,
}

impl UiPreparedScrollDirectSuccession {
    pub(crate) fn reserved_bytes(count: usize) -> Option<usize> {
        // Scroll and mounted pending maps, prepared frame and publication
        // receipt can coexist. Include both maps' keys and tree link/slack
        // allowance in the existing mounted retention reservation.
        let records = std::mem::size_of::<Self>().checked_mul(4)?;
        let map_entries = std::mem::size_of::<UiScrollOwnerIdentity>()
            .checked_add(6 * std::mem::size_of::<usize>())?
            .checked_mul(2)?;
        count.checked_mul(records.checked_add(map_entries)?)
    }

    pub(crate) const fn owner(self) -> UiScrollOwnerIdentity {
        self.owner
    }
    pub(crate) const fn occurrence(self) -> UiMountedInstanceIdentity {
        self.occurrence
    }
    pub(crate) const fn surface(self) -> UiSemanticSurfaceIdentity {
        self.owner.semantic_surface()
    }
    pub(crate) fn pose(
        self,
    ) -> Option<(
        UiSemanticSurfaceIdentity,
        UiMountedInstanceIdentity,
        crate::runtime::scroll::UiScrollOffset,
    )> {
        self.geometry_owner
            .map(|owner| (self.surface(), owner, self.record.offset))
    }
}

impl UiScrollRuntimeState {
    #[cfg(test)]
    pub(crate) fn pending_direct_count(&self) -> usize {
        self.pending_direct.len()
    }

    pub(crate) fn has_direct_succession(
        &self,
        prepared: &UiPreparedScrollDirectSuccession,
    ) -> bool {
        self.pending_direct.get(&prepared.owner) == Some(prepared)
    }

    pub(crate) fn retire_stale_direct_successions(&mut self) {
        let owners = &self.owners;
        self.pending_direct.retain(|owner, prepared| {
            owners
                .get(owner)
                .is_some_and(|current| current.incarnation == prepared.record.incarnation)
        });
    }

    pub(crate) fn prepare_direct_succession(
        &self,
        receipt: &UiScrollRouteReceipt,
        occurrence: UiMountedInstanceIdentity,
        geometry: &[Option<UiMountedInstanceIdentity>],
    ) -> Vec<UiPreparedScrollDirectSuccession> {
        assert!(receipt.transitions().len() <= geometry.len());
        receipt
            .transitions()
            .iter()
            .zip(geometry)
            .map(|(transition, geometry_owner)| {
                let owner = transition.owner();
                UiPreparedScrollDirectSuccession {
                    owner,
                    occurrence,
                    geometry_owner: *geometry_owner,
                    record: *self.owners.get(&owner).expect("routed owner exists"),
                    cause: receipt.cause(),
                    revision: receipt.revision(),
                }
            })
            .collect()
    }

    pub(crate) fn stage_direct_succession(
        &mut self,
        candidate: &Self,
        prepared: &[UiPreparedScrollDirectSuccession],
    ) {
        for record in prepared {
            self.pending_direct.insert(record.owner, *record);
        }
        // These count admitted input requests, not displayed effects. Offsets
        // and transition retirement remain behind physical acceptance below.
        self.counters = candidate.counters;
        self.revision = candidate.revision;
    }

    /// Lays a layout candidate for `surface` out from where direct input
    /// awaiting its frame put each owner there. Only a staged candidate
    /// adopts these records: the accepted state keeps them pending, and the
    /// frame that presents the layout commits the input first and the
    /// layout's records over it.
    pub(crate) fn adopt_pending_direct(&mut self, surface: UiSemanticSurfaceIdentity) {
        for (owner, prepared) in &self.pending_direct {
            if owner.semantic_surface() == surface
                && self
                    .owners
                    .get(owner)
                    .is_some_and(|current| current.incarnation == prepared.record.incarnation)
            {
                self.owners.insert(*owner, prepared.record);
            }
        }
    }

    /// The accepted offset of each owner that direct input on `surface` has
    /// staged past, keyed by the mounted owner its pose moves: where the
    /// frame the host shows still stands that owner.
    pub(crate) fn accepted_offsets_under_direct(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> impl Iterator<
        Item = (
            UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        ),
    > + '_ {
        self.pending_direct
            .iter()
            .filter(move |(owner, _)| owner.semantic_surface() == surface)
            .filter_map(|(owner, prepared)| {
                let (_, geometry_owner, _) = prepared.pose()?;
                self.owners
                    .get(owner)
                    .filter(|accepted| accepted.incarnation == prepared.record.incarnation)
                    .map(|accepted| (geometry_owner, accepted.offset))
            })
    }

    /// Forget the direct input staged on `surface` for a frame no binding
    /// will present. A surface that ends with no successor carries none of it
    /// to a later registration.
    pub(crate) fn retire_surface_direct_successions(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.pending_direct
            .retain(|owner, _| owner.semantic_surface() != surface);
    }

    pub(crate) fn has_pending_direct(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.pending_direct
            .keys()
            .any(|owner| owner.semantic_surface() == surface)
    }

    pub(crate) fn commit_presented_direct(
        &mut self,
        prepared: UiPreparedScrollDirectSuccession,
    ) -> bool {
        if self.pending_direct.get(&prepared.owner) != Some(&prepared)
            || self
                .owners
                .get(&prepared.owner)
                .is_none_or(|current| current.incarnation != prepared.record.incarnation)
        {
            return false;
        }
        self.owners.insert(prepared.owner, prepared.record);
        self.pending_direct.remove(&prepared.owner);
        self.retire_transition(prepared.owner);
        true
    }
}
