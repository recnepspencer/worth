//! Prepared layout offsets remain separate from accepted Scroll records until
//! the host accepts the surface whose content was lowered from those offsets.

use std::collections::BTreeMap;

use super::{UiScrollOwnerRecord, UiScrollRuntimeState};
use crate::runtime::scroll::UiScrollOwnerIdentity;
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
