use std::collections::HashMap;

use worth_ui_host_contract::{
    UiMountedPaintOrderEdit, UiMountedPaintOrderIdentity, UiMountedPaintOrderIntegrity,
};

use worth_ui_retained_order::UiRetainedOrderIndex;

pub(super) struct UiHeadlessRetainedOrder {
    index: UiRetainedOrderIndex<UiMountedPaintOrderIdentity>,
    integrity: UiMountedPaintOrderIntegrity,
}

pub(super) struct UiHeadlessRetainedOrderSnapshot {
    entries: Vec<OriginalOrderEntry>,
    integrity: UiMountedPaintOrderIntegrity,
}

struct OriginalOrderEntry {
    identity: UiMountedPaintOrderIdentity,
    placement: OriginalOrderPlacement<UiMountedPaintOrderIdentity>,
}

/// Where an identity stood before the transaction, if it was ordered at all.
enum OriginalOrderPlacement<Identity> {
    Unordered,
    Ranked {
        rank: usize,
        predecessor: Option<Identity>,
    },
}

impl<Identity> OriginalOrderPlacement<Identity> {
    const fn rank(&self) -> Option<usize> {
        match self {
            Self::Ranked { rank, .. } => Some(*rank),
            Self::Unordered => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiHeadlessRetainedOrderDenial {
    DuplicateIdentity,
    MissingIdentity,
    MissingPredecessor,
    SelfPredecessor,
    IntegrityMismatch,
    CapacityExceeded,
}

impl UiHeadlessRetainedOrder {
    pub(super) fn initial(
        identities: &[UiMountedPaintOrderIdentity],
        integrity: UiMountedPaintOrderIntegrity,
        capacity: usize,
    ) -> Result<Self, UiHeadlessRetainedOrderDenial> {
        if !integrity.admits(identities) {
            return Err(UiHeadlessRetainedOrderDenial::IntegrityMismatch);
        }
        let mut order = Self {
            index: UiRetainedOrderIndex::new(capacity),
            integrity,
        };
        let mut predecessor = None;
        for identity in identities {
            order.insert_after(*identity, predecessor)?;
            predecessor = Some(*identity);
        }
        Ok(order)
    }

    pub(super) fn apply(
        &mut self,
        edits: &[UiMountedPaintOrderEdit],
        expected: UiMountedPaintOrderIntegrity,
    ) -> Result<(), UiHeadlessRetainedOrderDenial> {
        for edit in edits {
            if edit.is_removal() {
                self.remove(edit.identity())?;
            } else {
                self.place_after(edit.identity(), edit.predecessor())?;
            }
        }
        if self.integrity != expected {
            return Err(UiHeadlessRetainedOrderDenial::IntegrityMismatch);
        }
        Ok(())
    }

    pub(super) fn contains(&self, identity: UiMountedPaintOrderIdentity) -> bool {
        self.index.contains(identity)
    }

    pub(super) fn take_cost(&self) -> worth_ui_retained_order::UiRetainedOrderCost {
        self.index.take_cost()
    }

    pub(super) fn snapshot(
        &self,
        identities: impl IntoIterator<Item = UiMountedPaintOrderIdentity>,
    ) -> UiHeadlessRetainedOrderSnapshot {
        let mut seen = HashMap::new();
        let mut entries = Vec::new();
        for identity in identities {
            if seen.insert(identity, ()).is_some() {
                continue;
            }
            let placement = match self.index.rank(identity) {
                Some(rank) => OriginalOrderPlacement::Ranked {
                    rank,
                    predecessor: rank
                        .checked_sub(1)
                        .and_then(|value| self.index.identity_at(value)),
                },
                None => OriginalOrderPlacement::Unordered,
            };
            entries.push(OriginalOrderEntry {
                identity,
                placement,
            });
        }
        entries.sort_unstable_by_key(|entry| entry.placement.rank());
        UiHeadlessRetainedOrderSnapshot {
            entries,
            integrity: self.integrity,
        }
    }

    pub(super) fn restore(
        &mut self,
        snapshot: UiHeadlessRetainedOrderSnapshot,
    ) -> Result<(), UiHeadlessRetainedOrderDenial> {
        for entry in &snapshot.entries {
            if self.contains(entry.identity) {
                self.index.remove(entry.identity);
            }
        }
        for entry in snapshot.entries {
            if let OriginalOrderPlacement::Ranked { predecessor, .. } = entry.placement {
                self.insert_after(entry.identity, predecessor)?;
            }
        }
        self.integrity = snapshot.integrity;
        Ok(())
    }

    fn remove(
        &mut self,
        identity: UiMountedPaintOrderIdentity,
    ) -> Result<(), UiHeadlessRetainedOrderDenial> {
        let (predecessor, successor) = self.neighbors(identity)?;
        let integrity = self
            .integrity
            .remove_edge(predecessor, identity, successor)
            .ok_or(UiHeadlessRetainedOrderDenial::IntegrityMismatch)?;
        if !self.index.remove(identity) {
            return Err(UiHeadlessRetainedOrderDenial::MissingIdentity);
        }
        self.integrity = integrity;
        Ok(())
    }

    fn place_after(
        &mut self,
        identity: UiMountedPaintOrderIdentity,
        predecessor: Option<UiMountedPaintOrderIdentity>,
    ) -> Result<(), UiHeadlessRetainedOrderDenial> {
        if predecessor == Some(identity) {
            return Err(UiHeadlessRetainedOrderDenial::SelfPredecessor);
        }
        if predecessor.is_some_and(|value| !self.index.contains(value)) {
            return Err(UiHeadlessRetainedOrderDenial::MissingPredecessor);
        }
        if self.contains(identity) {
            self.remove(identity)?;
        }
        let successor = self.successor_after(predecessor)?;
        let integrity = self
            .integrity
            .insert_edge(predecessor, identity, successor)
            .ok_or(UiHeadlessRetainedOrderDenial::IntegrityMismatch)?;
        self.insert_after(identity, predecessor)?;
        self.integrity = integrity;
        Ok(())
    }

    fn neighbors(
        &self,
        identity: UiMountedPaintOrderIdentity,
    ) -> Result<
        (
            Option<UiMountedPaintOrderIdentity>,
            Option<UiMountedPaintOrderIdentity>,
        ),
        UiHeadlessRetainedOrderDenial,
    > {
        let rank = self
            .index
            .rank(identity)
            .ok_or(UiHeadlessRetainedOrderDenial::MissingIdentity)?;
        Ok((
            rank.checked_sub(1)
                .and_then(|value| self.index.identity_at(value)),
            self.index.identity_at(rank + 1),
        ))
    }

    fn successor_after(
        &self,
        predecessor: Option<UiMountedPaintOrderIdentity>,
    ) -> Result<Option<UiMountedPaintOrderIdentity>, UiHeadlessRetainedOrderDenial> {
        match predecessor {
            Some(predecessor) => self
                .index
                .rank(predecessor)
                .map(|rank| self.index.identity_at(rank + 1))
                .ok_or(UiHeadlessRetainedOrderDenial::MissingPredecessor),
            None => Ok(self.index.identity_at(0)),
        }
    }

    fn insert_after(
        &mut self,
        identity: UiMountedPaintOrderIdentity,
        predecessor: Option<UiMountedPaintOrderIdentity>,
    ) -> Result<(), UiHeadlessRetainedOrderDenial> {
        if self.contains(identity) {
            return Err(UiHeadlessRetainedOrderDenial::DuplicateIdentity);
        }
        let rank = match predecessor {
            Some(predecessor) => self
                .index
                .rank(predecessor)
                .map(|rank| rank + 1)
                .ok_or(UiHeadlessRetainedOrderDenial::MissingPredecessor)?,
            None => 0,
        };
        self.index
            .insert_at(rank, identity)
            .map_err(|denial| match denial {
                worth_ui_retained_order::UiRetainedOrderDenial::CapacityExceeded => {
                    UiHeadlessRetainedOrderDenial::CapacityExceeded
                }
                worth_ui_retained_order::UiRetainedOrderDenial::DuplicateIdentity => {
                    UiHeadlessRetainedOrderDenial::DuplicateIdentity
                }
                worth_ui_retained_order::UiRetainedOrderDenial::InvalidRank => {
                    UiHeadlessRetainedOrderDenial::MissingPredecessor
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{UiHeadlessRetainedOrder, UiHeadlessRetainedOrderDenial};
    use worth_ui_host_contract::{
        UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
        UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer,
        UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity, UiMountedPaintOrderIntegrity,
        UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionInput,
        UiMountedPortalOverlayLifecyclePosture, UiMountedPortalOverlayMechanic, UiMountedRgba8,
        UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    };

    const PROFILE_CAPACITY: usize = 4_096;

    #[test]
    fn profile_ceiling_denies_4097_without_mutating_order_integrity_or_high_water() {
        let identities = (0..u64::try_from(PROFILE_CAPACITY).unwrap())
            .map(order_identity)
            .collect::<Vec<_>>();
        let integrity = UiMountedPaintOrderIntegrity::for_order(&identities);
        let mut order =
            UiHeadlessRetainedOrder::initial(&identities, integrity, PROFILE_CAPACITY).unwrap();
        let expected = order.index.ordered().collect::<Vec<_>>();
        order.take_cost();

        assert_eq!(
            order.place_after(order_identity(4_097), Some(identities[0])),
            Err(UiHeadlessRetainedOrderDenial::CapacityExceeded)
        );
        assert_eq!(order.integrity, integrity);
        let denied = order.take_cost();
        assert_eq!(denied.live_entries(), 4_096);
        assert_eq!(denied.allocated_slots(), 4_096);
        assert_eq!(denied.high_water_entries(), 4_096);
        assert_eq!(denied.rotations(), 0);
        assert_eq!(order.index.ordered().collect::<Vec<_>>(), expected);
    }

    fn order_identity(slot: u64) -> UiMountedPaintOrderIdentity {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: slot as f32,
            y: 0.0,
            width: 1.0,
            height: 1.0,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap();
        let mechanic = UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame,
                surface,
                binding,
                owner: instance,
                owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(instance),
                portal_identity: instance.diagnostic_value(),
                anchor_presentation: UiHostObservationPresentationBasis::new(
                    UiHostSurfaceIdentity::mint_unbound().unwrap(),
                    frame,
                    binding,
                    UiHostPresentationEpoch::issued_by_host(1),
                ),
                anchor_bounds: bounds,
                bounds,
                paint_bounds: bounds,
                color: UiMountedRgba8::new(1, 2, 3, 255),
                layer_semantic_order: 0,
                layer_depth: 0,
                clip_bounds: bounds,
                lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
                shielding: UiMountedPortalInputShielding::ContentBounds,
            },
        )
        .unwrap();
        UiMountedPaintOrderIdentity::for_command(UiMountedPaintCommandIdentity::portal_overlay(
            &mechanic,
        ))
    }
}
