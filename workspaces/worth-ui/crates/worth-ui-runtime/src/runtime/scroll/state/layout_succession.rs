//! Prepared layout offsets remain separate from accepted Scroll records until
//! the host accepts the surface whose content was lowered from those offsets.
//!
//! A layout is lowered from the offsets Scroll accepted when it was staged.
//! A settle that arrives before the layout is shown shows its content past
//! them, so the staged record follows it there, within the staged bounds.

use std::collections::BTreeMap;

use super::{UiScrollOwnerRecord, UiScrollRuntimeState};
use crate::runtime::scroll::{UiScrollOffset, UiScrollOwnerIdentity, UiScrollOwnerIncarnation};
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

#[derive(Clone)]
pub(super) struct UiScrollLayoutSuccessor {
    owners: BTreeMap<UiScrollOwnerIdentity, UiScrollOwnerRecord>,
}

impl UiScrollRuntimeState {
    pub(super) fn retire_layout_owner(&mut self, owner: UiScrollOwnerIdentity) {
        self.pending_direct.remove(&owner);
        if let Some(candidate) = self.pending_layouts.get_mut(&owner.semantic_surface()) {
            candidate.owners.remove(&owner);
        }
    }

    pub(crate) fn stage_layout_successor(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
        candidate: &Self,
    ) {
        // Ownership is layout identity, not a displayed offset. The new
        // occurrence/chain index is needed to lower chrome for this candidate;
        // existing accepted owner records remain untouched until publication.
        self.ownership_catalog = candidate.ownership_catalog.clone();
        self.ownership_references = candidate.ownership_references.clone();
        let owners = candidate
            .owners
            .iter()
            .filter(|(owner, _)| owner.semantic_surface() == surface)
            .map(|(owner, record)| (*owner, *record))
            .collect();
        self.pending_layouts
            .insert(surface, UiScrollLayoutSuccessor { owners });
    }

    pub(crate) fn commit_presented_layout(&mut self, surface: UiSemanticSurfaceIdentity) -> bool {
        let Some(candidate) = self.pending_layouts.remove(&surface) else {
            return false;
        };
        for (owner, record) in candidate.owners {
            if !self.ownership_references.contains_key(&owner) {
                continue;
            }
            self.owners.insert(owner, record);
            self.reconcile_transition_bounds(owner, record.incarnation, record.bounds);
        }
        true
    }

    /// Where the layout staged for `owner`'s surface would move this
    /// incarnation of `owner` to follow `displayed`: that offset within the
    /// staged bounds. `None` when no staged layout holds it or it already
    /// stands there.
    pub(crate) fn staged_layout_offset(
        &self,
        owner: UiScrollOwnerIdentity,
        incarnation: UiScrollOwnerIncarnation,
        displayed: UiScrollOffset,
    ) -> Option<UiScrollOffset> {
        self.pending_layouts
            .get(&owner.semantic_surface())?
            .owners
            .get(&owner)
            .filter(|record| record.incarnation == incarnation)
            .map(|record| (record.offset, record.bounds.clamp(displayed)))
            .and_then(|(stands, follows)| (stands != follows).then_some(follows))
    }

    /// Stand this incarnation of `owner` in its surface's staged layout
    /// where `staged_layout_offset` answered for `displayed`.
    pub(crate) fn settle_staged_layout(
        &mut self,
        owner: UiScrollOwnerIdentity,
        incarnation: UiScrollOwnerIncarnation,
        displayed: UiScrollOffset,
    ) {
        if let Some(record) = self
            .pending_layouts
            .get_mut(&owner.semantic_surface())
            .and_then(|candidate| candidate.owners.get_mut(&owner))
            .filter(|record| record.incarnation == incarnation)
        {
            record.offset = record.bounds.clamp(displayed);
        }
    }

    /// Whether direct input awaiting its frame has staged `owner` past its
    /// accepted offset.
    pub(crate) fn has_pending_direct_owner(&self, owner: UiScrollOwnerIdentity) -> bool {
        self.pending_direct.contains_key(&owner)
    }

    pub(crate) fn has_unpresented_layout(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.pending_layouts.contains_key(&surface)
    }

    pub(crate) fn accepted_owner_bounds(
        &self,
        owner: UiScrollOwnerIdentity,
        incarnation: crate::runtime::scroll::UiScrollOwnerIncarnation,
    ) -> Result<crate::runtime::scroll::UiScrollBounds, crate::runtime::scroll::UiScrollRouteDenial>
    {
        self.exact_owner(owner, incarnation)
            .map(|record| record.bounds)
    }
}
