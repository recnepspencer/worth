use std::collections::{BTreeSet, HashSet};

use super::{
    UiAppearanceDamageRegion, UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearanceMechanicIdentity, UiMountedOverlayOrderMechanic,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedAppearanceFrame {
    frame: crate::UiMountedFrameIdentity,
    semantic_surface: crate::UiSemanticSurfaceIdentity,
    mechanics: Box<[UiMountedAppearanceMechanic]>,
    overlay_order: UiMountedOverlayOrderMechanic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UiMountedAppearanceFrameDenial {
    DuplicateMechanic(UiMountedAppearanceMechanicIdentity),
    NodeReceiptFrameMismatch,
    MechanicSurfaceMismatch,
    OverlaySurfaceMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedAppearancePredecessorManifest {
    mechanic_identities: Box<[UiMountedAppearanceMechanicIdentity]>,
    overlay_order: Box<[crate::UiOverlayParticipantIdentity]>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedAppearanceWorkPosture {
    Initial,
    Delta,
    Reconstruction,
    Unchanged,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiMountedAppearanceWork {
    posture: UiMountedAppearanceWorkPosture,
    predecessor: Option<crate::UiMountedFrameIdentity>,
    predecessor_manifest: Option<UiMountedAppearancePredecessorManifest>,
    successor: UiMountedAppearanceFrame,
    changes: Box<[UiMountedAppearanceMechanicChange]>,
    damage: Box<[UiAppearanceDamageRegion]>,
    order_changed: bool,
}

impl UiMountedAppearancePredecessorManifest {
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        mechanic_identities: impl IntoIterator<Item = UiMountedAppearanceMechanicIdentity>,
        overlay_order: impl IntoIterator<Item = crate::UiOverlayParticipantIdentity>,
    ) -> Option<Self> {
        let mechanic_identities = mechanic_identities.into_iter().collect::<Vec<_>>();
        let overlay_order = overlay_order.into_iter().collect::<Vec<_>>();
        let mut mechanics_seen = HashSet::new();
        if mechanic_identities
            .iter()
            .any(|identity| !mechanics_seen.insert(identity.clone()))
        {
            return None;
        }
        let mut order_seen = BTreeSet::new();
        if overlay_order
            .iter()
            .any(|participant| !order_seen.insert(participant.clone()))
        {
            return None;
        }
        Some(Self {
            mechanic_identities: mechanic_identities.into_boxed_slice(),
            overlay_order: overlay_order.into_boxed_slice(),
        })
    }

    pub fn mechanic_identities(&self) -> &[UiMountedAppearanceMechanicIdentity] {
        &self.mechanic_identities
    }

    pub fn overlay_order(&self) -> &[crate::UiOverlayParticipantIdentity] {
        &self.overlay_order
    }
}

impl UiMountedAppearanceFrame {
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        frame: crate::UiMountedFrameIdentity,
        semantic_surface: crate::UiSemanticSurfaceIdentity,
        mechanics: impl IntoIterator<Item = UiMountedAppearanceMechanic>,
        overlay_order: UiMountedOverlayOrderMechanic,
    ) -> Result<Self, UiMountedAppearanceFrameDenial> {
        if overlay_order.semantic_surface() != semantic_surface {
            return Err(UiMountedAppearanceFrameDenial::OverlaySurfaceMismatch);
        }
        let mechanics = mechanics.into_iter().collect::<Vec<_>>();
        let mut seen = HashSet::new();
        if let Some(duplicate) = mechanics
            .iter()
            .map(UiMountedAppearanceMechanic::identity)
            .find(|identity| !seen.insert(identity.clone()))
        {
            return Err(UiMountedAppearanceFrameDenial::DuplicateMechanic(duplicate));
        }
        if mechanics.iter().any(|mechanic| {
            mechanic
                .node_receipt_frame()
                .is_some_and(|receipt_frame| receipt_frame != frame)
        }) {
            return Err(UiMountedAppearanceFrameDenial::NodeReceiptFrameMismatch);
        }
        if mechanics.iter().any(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Pointer(mechanic) => {
                mechanic.surface() != semantic_surface
            }
            UiMountedAppearanceMechanic::Backdrop(mechanic) => {
                mechanic.semantic_surface() != semantic_surface
            }
            _ => false,
        }) {
            return Err(UiMountedAppearanceFrameDenial::MechanicSurfaceMismatch);
        }
        Ok(Self {
            frame,
            semantic_surface,
            mechanics: mechanics.into_boxed_slice(),
            overlay_order,
        })
    }

    pub const fn frame(&self) -> crate::UiMountedFrameIdentity {
        self.frame
    }
    pub const fn semantic_surface(&self) -> crate::UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub fn mechanics(&self) -> &[UiMountedAppearanceMechanic] {
        &self.mechanics
    }
    pub const fn overlay_order(&self) -> &UiMountedOverlayOrderMechanic {
        &self.overlay_order
    }
    pub const fn is_unpublished(&self) -> bool {
        true
    }
    pub const fn live_host_command_count(&self) -> u32 {
        0
    }
}

impl UiMountedAppearanceWork {
    #[doc(hidden)]
    pub fn from_runtime_mounting(
        posture: UiMountedAppearanceWorkPosture,
        predecessor: Option<crate::UiMountedFrameIdentity>,
        predecessor_manifest: Option<UiMountedAppearancePredecessorManifest>,
        successor: UiMountedAppearanceFrame,
        changes: impl IntoIterator<Item = UiMountedAppearanceMechanicChange>,
        damage: impl IntoIterator<Item = UiAppearanceDamageRegion>,
        order_changed: bool,
    ) -> Option<Self> {
        let changes = changes.into_iter().collect::<Vec<_>>();
        let damage = damage.into_iter().collect::<Vec<_>>();
        if predecessor.is_some() != predecessor_manifest.is_some()
            || !changes_are_structurally_valid(
                posture,
                predecessor_manifest.as_ref(),
                &successor,
                &changes,
            )
        {
            return None;
        }
        let actual_order_changed = predecessor_manifest.as_ref().map_or(true, |manifest| {
            manifest.overlay_order() != successor.overlay_order().bottom_to_top()
        });
        if order_changed != actual_order_changed {
            return None;
        }
        match posture {
            UiMountedAppearanceWorkPosture::Initial if predecessor.is_some() => return None,
            UiMountedAppearanceWorkPosture::Delta
            | UiMountedAppearanceWorkPosture::Reconstruction
            | UiMountedAppearanceWorkPosture::Unchanged
                if predecessor.is_none() =>
            {
                return None
            }
            UiMountedAppearanceWorkPosture::Initial if !order_changed => return None,
            UiMountedAppearanceWorkPosture::Delta if changes.is_empty() && !order_changed => {
                return None;
            }
            UiMountedAppearanceWorkPosture::Unchanged
                if !changes.is_empty() || !damage.is_empty() || order_changed =>
            {
                return None;
            }
            _ => {}
        }
        Some(Self {
            posture,
            predecessor,
            predecessor_manifest,
            successor,
            changes: changes.into_boxed_slice(),
            damage: damage.into_boxed_slice(),
            order_changed,
        })
    }

    pub const fn posture(&self) -> UiMountedAppearanceWorkPosture {
        self.posture
    }
    pub const fn predecessor(&self) -> Option<crate::UiMountedFrameIdentity> {
        self.predecessor
    }
    pub const fn predecessor_manifest(&self) -> Option<&UiMountedAppearancePredecessorManifest> {
        self.predecessor_manifest.as_ref()
    }
    pub const fn successor(&self) -> &UiMountedAppearanceFrame {
        &self.successor
    }
    pub fn changes(&self) -> &[UiMountedAppearanceMechanicChange] {
        &self.changes
    }
    pub fn damage(&self) -> &[UiAppearanceDamageRegion] {
        &self.damage
    }
    pub const fn order_changed(&self) -> bool {
        self.order_changed
    }
    pub const fn live_host_command_count(&self) -> u32 {
        0
    }
}

fn changes_are_structurally_valid(
    posture: UiMountedAppearanceWorkPosture,
    predecessor_manifest: Option<&UiMountedAppearancePredecessorManifest>,
    successor: &UiMountedAppearanceFrame,
    changes: &[UiMountedAppearanceMechanicChange],
) -> bool {
    let mut identities = HashSet::new();
    if changes.iter().any(|change| {
        let identity = match change {
            UiMountedAppearanceMechanicChange::Insert(mechanic) => mechanic.identity(),
            UiMountedAppearanceMechanicChange::Replace { predecessor, .. }
            | UiMountedAppearanceMechanicChange::Remove(predecessor) => predecessor.clone(),
        };
        !identities.insert(identity)
    }) {
        return false;
    }
    match posture {
        UiMountedAppearanceWorkPosture::Initial => {
            predecessor_manifest.is_none()
                && changes.len() == successor.mechanics().len()
                && changes.iter().all(|change| {
                    matches!(change, UiMountedAppearanceMechanicChange::Insert(mechanic)
                        if successor.mechanics().iter().any(|candidate| candidate == mechanic))
                })
        }
        UiMountedAppearanceWorkPosture::Delta
        | UiMountedAppearanceWorkPosture::Reconstruction
        | UiMountedAppearanceWorkPosture::Unchanged => {
            let Some(predecessor_manifest) = predecessor_manifest else {
                return false;
            };
            let changes_valid = changes.iter().all(|change| match change {
                UiMountedAppearanceMechanicChange::Insert(mechanic) => {
                    !predecessor_manifest
                        .mechanic_identities()
                        .contains(&mechanic.identity())
                        && successor
                            .mechanics()
                            .iter()
                            .any(|candidate| candidate == mechanic)
                }
                UiMountedAppearanceMechanicChange::Replace {
                    predecessor,
                    successor: mechanic,
                } => {
                    predecessor == &mechanic.identity()
                        && predecessor_manifest
                            .mechanic_identities()
                            .contains(predecessor)
                        && successor
                            .mechanics()
                            .iter()
                            .any(|candidate| candidate == mechanic)
                }
                UiMountedAppearanceMechanicChange::Remove(identity) => {
                    predecessor_manifest
                        .mechanic_identities()
                        .contains(identity)
                        && !successor
                            .mechanics()
                            .iter()
                            .any(|mechanic| mechanic.identity() == *identity)
                }
            });
            let identity_sets_are_covered = predecessor_manifest
                .mechanic_identities()
                .iter()
                .all(|identity| {
                    successor
                        .mechanics()
                        .iter()
                        .any(|mechanic| mechanic.identity() == *identity)
                        || changes.iter().any(|change| {
                            matches!(change, UiMountedAppearanceMechanicChange::Remove(candidate) if candidate == identity)
                        })
                })
                && successor.mechanics().iter().all(|mechanic| {
                    predecessor_manifest
                        .mechanic_identities()
                        .contains(&mechanic.identity())
                        || changes.iter().any(|change| {
                            matches!(change, UiMountedAppearanceMechanicChange::Insert(candidate) if candidate == mechanic)
                        })
                });
            changes_valid && identity_sets_are_covered
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_denies_a_pointer_from_another_semantic_surface() {
        let frame = crate::UiMountedFrameIdentity::mint_unbound().unwrap();
        let surface = crate::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let foreign_surface = crate::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let pointer = UiMountedAppearanceMechanic::Pointer(
            crate::UiMountedPointerAffordanceMechanic::complete_from_runtime_mounting(
                crate::UiHostPointerIdentity::new(1),
                foreign_surface,
                crate::UiMountedInstanceIdentity::mint_unbound().unwrap(),
                crate::UiPointerAffordanceFamily::Default,
            ),
        );
        let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
            surface,
            crate::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            1,
            1,
            [],
        )
        .unwrap();

        assert_eq!(
            UiMountedAppearanceFrame::from_runtime_mounting(frame, surface, [pointer], order),
            Err(UiMountedAppearanceFrameDenial::MechanicSurfaceMismatch)
        );
    }
}
