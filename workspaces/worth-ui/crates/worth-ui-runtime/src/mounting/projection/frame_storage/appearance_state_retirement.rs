use crate::runtime::persistent_index::UiPersistentOrdMap;

use super::super::appearance::UiMountedAppearanceLoweringDenial;
use super::appearance_output::UiMountedAppearanceNodeWork;
use super::appearance_state_membership::UiMountedAppearanceLocalNodeKey;
use super::appearance_state_membership_work::UiMountedAppearanceMembershipWork;
use super::appearance_state_predecessor::UiMountedAppearancePhysicalPredecessor;

/// Explicit removals from the last accepted physical set. Candidate admission
/// drains this set atomically; rejected candidates cannot add physical rows.
#[derive(Clone, Default)]
pub(super) struct UiMountedAppearanceRetirements {
    pending:
        UiPersistentOrdMap<UiMountedAppearanceLocalNodeKey, UiMountedAppearancePhysicalPredecessor>,
}

impl UiMountedAppearanceRetirements {
    pub(super) fn forget_surface(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> UiMountedAppearanceMembershipWork {
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(self.pending.len());
        let removed = self
            .pending
            .iter()
            .filter_map(|(key, _)| (key.surface == surface).then_some(key.clone()))
            .collect::<Vec<_>>();
        for key in removed {
            let (_, mutation) = self.pending.remove_with_work(&key);
            work.add_mutation(mutation);
        }
        work
    }

    pub(super) fn capture(
        &mut self,
        physical: UiMountedAppearancePhysicalPredecessor,
    ) -> UiMountedAppearanceMembershipWork {
        let mut work = UiMountedAppearanceMembershipWork::default();
        let (existing, probes) = self.pending.get_with_probes(&physical.key);
        work.add_lookup(probes);
        assert!(
            existing.is_none(),
            "one accepted physical predecessor per mounted identity"
        );
        work.add_mutation(
            self.pending
                .insert_with_work(physical.key.clone(), physical),
        );
        work
    }

    pub(super) fn lower(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> (
        Result<Vec<UiMountedAppearanceNodeWork>, UiMountedAppearanceLoweringDenial>,
        UiMountedAppearanceMembershipWork,
    ) {
        let mut work = UiMountedAppearanceMembershipWork::default();
        let mut removals = Vec::with_capacity(self.pending.len());
        for (_, predecessor) in self.pending.iter() {
            work.add_traversal(1);
            if predecessor.session != session {
                return (
                    Err(UiMountedAppearanceLoweringDenial::NodeSessionMismatch),
                    work,
                );
            }
            let removal = match predecessor.sidecar.removal_work(frame, presentation) {
                Ok(removal) => removal,
                Err(denial) => return (Err(denial), work),
            };
            removals.push(UiMountedAppearanceNodeWork {
                predecessor: predecessor.sidecar.current_node_receipt(),
                successor: None,
                work: removal,
            });
        }
        self.pending = UiPersistentOrdMap::default();
        (Ok(removals), work)
    }

    #[cfg(test)]
    pub(super) fn receipts(
        &self,
    ) -> impl Iterator<Item = worth_ui_host_contract::UiMountedNodeReceiptIdentity> + '_ {
        self.pending
            .iter()
            .filter_map(|(_, predecessor)| predecessor.sidecar.current_node_receipt())
    }
}
