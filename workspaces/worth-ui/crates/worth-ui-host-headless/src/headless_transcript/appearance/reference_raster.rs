use worth_ui_host_contract::{
    compose_source_over, UiMountedAppearanceColor, UiMountedAppearanceFrame,
    UiMountedAppearanceMechanic, UiMountedSurfacePaint, UiOverlayParticipantIdentity,
};

/// Returns the deterministic interior sample for the issued overlay stack.
///
/// This is a headless reference value only. It is derived from the mounted
/// mechanics and issued order; it cannot publish a frame or authorize host
/// work.
pub(crate) fn compose(frame: &UiMountedAppearanceFrame) -> Option<UiMountedAppearanceColor> {
    let mut layers = Vec::new();
    for participant in frame.overlay_order().bottom_to_top() {
        let mechanic = frame
            .mechanics()
            .iter()
            .find(|mechanic| match participant {
                UiOverlayParticipantIdentity::Portal(instance) => matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::PortalSurface(surface)
                        if surface.portal_instance() == *instance
                ),
                UiOverlayParticipantIdentity::Backdrop(identity) => matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::Backdrop(backdrop)
                        if backdrop.identity() == identity
                ),
            })?;
        match mechanic {
            UiMountedAppearanceMechanic::PortalSurface(surface) => layers.push((
                surface_color(surface.surface().paint()),
                surface.surface().opacity(),
            )),
            UiMountedAppearanceMechanic::Backdrop(backdrop) => {
                layers.push((backdrop.background(), backdrop.opacity()))
            }
            _ => {}
        }
    }
    Some(compose_source_over(layers))
}

pub(crate) fn has_issued_participants(frame: &UiMountedAppearanceFrame) -> bool {
    frame
        .overlay_order()
        .bottom_to_top()
        .iter()
        .enumerate()
        .all(|(ordinal, participant)| {
            frame.mechanics().iter().any(|mechanic| match participant {
                UiOverlayParticipantIdentity::Portal(instance) => matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::PortalSurface(surface)
                        if surface.portal_instance() == *instance
                ),
                UiOverlayParticipantIdentity::Backdrop(identity) => matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::Backdrop(backdrop)
                        if backdrop.identity() == identity
                            && backdrop.placement().overlay_revision()
                                == frame.overlay_order().portal_revision()
                            && usize::try_from(backdrop.placement().ordinal()) == Ok(ordinal)
                ),
            })
        })
        && frame
            .mechanics()
            .iter()
            .filter_map(|mechanic| match mechanic {
                UiMountedAppearanceMechanic::PortalSurface(surface) => Some(
                    UiOverlayParticipantIdentity::Portal(surface.portal_instance()),
                ),
                UiMountedAppearanceMechanic::Backdrop(backdrop) => Some(
                    UiOverlayParticipantIdentity::Backdrop(backdrop.identity().clone()),
                ),
                _ => None,
            })
            .all(|participant| frame.overlay_order().bottom_to_top().contains(&participant))
}

fn surface_color(paint: &UiMountedSurfacePaint) -> UiMountedAppearanceColor {
    match paint {
        UiMountedSurfacePaint::Fill(color) | UiMountedSurfacePaint::Border { color, .. } => *color,
        UiMountedSurfacePaint::FillAndBorder { fill, .. } => *fill,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn portal(
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        color: [u8; 4],
    ) -> UiMountedAppearanceMechanic {
        let issuer = worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let bounds = worth_ui_host_contract::UiAppearanceAllocationBounds::new(0, 0, 8, 8).unwrap();
        let surface = worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic::
            complete_from_runtime_mounting(
                worth_ui_host_contract::UiMountedSurfaceAppearanceCompletionInput {
                    issuer,
                    node_receipt: issuer.receipt_for(instance),
                    bounds,
                    clip: worth_ui_host_contract::UiAppearanceClip::new(0, 0, 8, 8).unwrap(),
                    layer: worth_ui_host_contract::UiMountedLayerProjection::Layer(
                        worth_ui_host_contract::UiMountedLayerReference::new(0),
                    ),
                    radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
                        bounds,
                        [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
                    ),
                    paint: UiMountedSurfacePaint::Fill(
                        UiMountedAppearanceColor::from_straight_srgba(color),
                    ),
                    opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
                    projection: worth_ui_host_contract::UiMountedNodeAppearanceAttribution::
                        from_runtime_mounting(issuer, 1, 1)
                        .unwrap(),
                },
            )
            .unwrap();
        UiMountedAppearanceMechanic::PortalSurface(
            worth_ui_host_contract::UiMountedPortalSurfaceAppearanceMechanic::
                complete_from_runtime_mounting(instance, surface)
                .unwrap(),
        )
    }

    #[test]
    fn issued_backdrop_then_portal_uses_deterministic_source_over() {
        let surface = worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let presentation =
            worth_ui_host_contract::UiMountedPresentationAttemptIdentity::mint_unbound().unwrap();
        let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let placement =
            worth_ui_host_contract::UiOverlayPlacementReceipt::from_runtime_overlay_order(3, 0)
                .unwrap();
        let identity = worth_ui_host_contract::UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.backdrop",
            worth_ui_host_contract::UiMountedBackdropScope::PerPortalInstance(instance),
            1,
        )
        .unwrap();
        let backdrop = worth_ui_host_contract::UiMountedBackdropMechanic::
            complete_from_runtime_mounting(
                worth_ui_host_contract::UiMountedBackdropCompletionInput {
                    identity: identity.clone(),
                    semantic_surface: surface,
                    placement,
                    extent: worth_ui_host_contract::UiAppearanceBackdropExtent::new(
                        0, 0, 8, 8,
                    )
                    .unwrap(),
                    clip: worth_ui_host_contract::UiAppearanceClip::new(0, 0, 8, 8).unwrap(),
                    background: UiMountedAppearanceColor::from_straight_srgba(
                        [255, 0, 0, 128],
                    ),
                    opacity: worth_ui_host_contract::UiMountedAppearanceOpacity::ONE,
                    attribution: worth_ui_host_contract::UiMountedBackdropAppearanceAttribution::
                        from_runtime_transport(surface, placement, 1, 1)
                        .unwrap(),
                },
            )
            .unwrap();
        let order = worth_ui_host_contract::UiMountedOverlayOrderMechanic::
            complete_from_runtime_overlay_order(
                surface,
                presentation,
                3,
                4,
                [
                    UiOverlayParticipantIdentity::Backdrop(identity),
                    UiOverlayParticipantIdentity::Portal(instance),
                ],
            )
            .unwrap();
        let frame_identity =
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let frame = UiMountedAppearanceFrame::from_runtime_mounting(
            frame_identity,
            surface,
            [
                UiMountedAppearanceMechanic::Backdrop(backdrop),
                portal(frame_identity, instance, [0, 255, 0, 128]),
            ],
            order,
        )
        .unwrap();

        assert_eq!(
            compose(&frame).unwrap().straight_srgba(),
            [156, 213, 0, 192]
        );
    }
}
