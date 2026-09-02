use worth_ui_host_contract::{
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange, UiMountedAppearanceWork,
};

pub(super) fn validate_changes(source: &UiMountedAppearanceWork) -> bool {
    let predecessor_manifest = source.predecessor_manifest();
    let mut identities = Vec::with_capacity(source.changes().len());
    let changes_are_valid = source.changes().iter().all(|change| {
        let (identity, valid) = match change {
            UiMountedAppearanceMechanicChange::Insert(mechanic) => (
                mechanic.identity(),
                validate(mechanic)
                    && successor_contains(source, mechanic)
                    && predecessor_manifest.is_none_or(|manifest| {
                        !manifest
                            .mechanic_identities()
                            .contains(&mechanic.identity())
                    }),
            ),
            UiMountedAppearanceMechanicChange::Replace {
                predecessor,
                successor,
            } => (
                predecessor.clone(),
                predecessor == &successor.identity()
                    && predecessor_manifest.is_some_and(|manifest| {
                        manifest.mechanic_identities().contains(predecessor)
                    })
                    && validate(successor)
                    && successor_contains(source, successor),
            ),
            UiMountedAppearanceMechanicChange::Remove(identity) => (
                identity.clone(),
                predecessor_manifest
                    .is_some_and(|manifest| manifest.mechanic_identities().contains(identity))
                    && !successor_contains_identity(source, identity),
            ),
        };
        let unique = !identities.iter().any(|candidate| candidate == &identity);
        identities.push(identity);
        unique && valid
    });
    changes_are_valid
        && match source.posture() {
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Initial => {
                source.predecessor().is_none()
                    && source
                        .changes()
                        .iter()
                        .all(|change| matches!(change, UiMountedAppearanceMechanicChange::Insert(_)))
                    && source.successor().mechanics().iter().all(|mechanic| {
                        source.changes().iter().any(|change| {
                            matches!(change, UiMountedAppearanceMechanicChange::Insert(candidate) if candidate == mechanic)
                        })
                    })
            }
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Unchanged => {
                predecessor_manifest.is_some_and(|manifest| {
                    source.predecessor().is_some()
                        && source.changes().is_empty()
                        && source.damage().is_empty()
                        && !source.order_changed()
                        && manifest.overlay_order()
                            == source.successor().overlay_order().bottom_to_top()
                        && manifest.mechanic_identities().len()
                            == source.successor().mechanics().len()
                        && source.successor().mechanics().iter().all(|mechanic| {
                            manifest.mechanic_identities().contains(&mechanic.identity())
                        })
                })
            }
            worth_ui_host_contract::UiMountedAppearanceWorkPosture::Delta
            | worth_ui_host_contract::UiMountedAppearanceWorkPosture::Reconstruction => {
                let Some(manifest) = predecessor_manifest else {
                    return false;
                };
                source.predecessor().is_some()
                    && source.order_changed()
                        == (manifest.overlay_order()
                            != source.successor().overlay_order().bottom_to_top())
                    && identity_sets_are_covered(source, manifest)
            }
        }
}

fn identity_sets_are_covered(
    source: &UiMountedAppearanceWork,
    predecessor_manifest: &worth_ui_host_contract::UiMountedAppearancePredecessorManifest,
) -> bool {
    predecessor_manifest
        .mechanic_identities()
        .iter()
        .all(|identity| {
            successor_contains_identity(source, identity)
                || source.changes().iter().any(|change| {
                    matches!(change, UiMountedAppearanceMechanicChange::Remove(candidate) if candidate == identity)
                })
        })
        && source.successor().mechanics().iter().all(|mechanic| {
            predecessor_manifest
                .mechanic_identities()
                .contains(&mechanic.identity())
                || source.changes().iter().any(|change| {
                    matches!(change, UiMountedAppearanceMechanicChange::Insert(candidate) if candidate == mechanic)
                })
        })
}

fn successor_contains(
    source: &UiMountedAppearanceWork,
    mechanic: &UiMountedAppearanceMechanic,
) -> bool {
    source
        .successor()
        .mechanics()
        .iter()
        .any(|candidate| candidate == mechanic)
}

fn successor_contains_identity(
    source: &UiMountedAppearanceWork,
    identity: &worth_ui_host_contract::UiMountedAppearanceMechanicIdentity,
) -> bool {
    source
        .successor()
        .mechanics()
        .iter()
        .any(|mechanic| mechanic.identity() == *identity)
}

fn validate(mechanic: &UiMountedAppearanceMechanic) -> bool {
    match mechanic {
        UiMountedAppearanceMechanic::Surface(mechanic) => {
            mechanic.visual_bounds()
                == worth_ui_host_contract::UiAppearanceVisualBounds::from_surface_allocation(
                    mechanic.bounds(),
                )
        }
        UiMountedAppearanceMechanic::PortalSurface(mechanic) => {
            mechanic.portal_instance() == mechanic.surface().node_receipt().mounted_instance()
        }
        UiMountedAppearanceMechanic::Outline(mechanic) => !mechanic.participates_in_hit_testing(),
        UiMountedAppearanceMechanic::TextForeground(mechanic) => {
            mechanic
                .node_receipt()
                .mounted_instance()
                .diagnostic_value()
                != 0
        }
        UiMountedAppearanceMechanic::Pointer(mechanic) => mechanic.pointer().value() != 0,
        UiMountedAppearanceMechanic::Backdrop(mechanic) => !mechanic.participates_in_hit_testing(),
    }
}
