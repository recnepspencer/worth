use worth_ui_host_contract::{
    UiMountedAppearanceFrame, UiMountedAppearanceMechanic, UiOverlayParticipantIdentity,
};

use super::fact::{
    UiMountedAppearanceFact, UiMountedAppearanceFacts, UiMountedAppearanceLoweringInput,
    UiMountedAppearanceNodeInput, UiMountedAppearanceVisualBounds,
};
use super::{
    backdrop, outline, overlay_order, pointer_affordance, surface, text_foreground,
    UiMountedAppearanceLoweringDenial,
};

pub(super) fn lower(
    input: UiMountedAppearanceLoweringInput,
) -> Result<UiMountedAppearanceFacts, UiMountedAppearanceLoweringDenial> {
    if input.overlay.semantic_surface != input.semantic_surface {
        return Err(UiMountedAppearanceLoweringDenial::NodeSurfaceMismatch);
    }
    if input.overlay.presentation != input.presentation {
        return Err(UiMountedAppearanceLoweringDenial::OverlayRevisionMissing);
    }
    let mut mechanics = Vec::new();
    let mut records = Vec::new();
    for node in &input.nodes {
        lower_node(&input, node, &mut mechanics, &mut records)?;
    }
    for backdrop_input in &input.backdrops {
        if backdrop_input.semantic_surface != input.semantic_surface {
            return Err(UiMountedAppearanceLoweringDenial::NodeSurfaceMismatch);
        }
        let mechanic = backdrop::lower(backdrop_input)?;
        records.push(UiMountedAppearanceFact::backdrop(
            input.semantic_surface,
            backdrop_input.attribution,
            backdrop_input.semantic_digest,
            mechanic.clone(),
            backdrop_input.extent,
        ));
        mechanics.push(mechanic);
    }
    let overlay_order = overlay_order::lower(&input.overlay)?;
    require_issued_participants(&overlay_order, &mechanics)?;
    validate_backdrop_placements(&overlay_order, &mechanics)?;
    let frame = UiMountedAppearanceFrame::from_runtime_mounting(
        input.frame,
        input.semantic_surface,
        mechanics,
        overlay_order,
    )
    .map_err(UiMountedAppearanceLoweringDenial::Frame)?;
    Ok(UiMountedAppearanceFacts::new(frame, records))
}

fn validate_backdrop_placements(
    order: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    mechanics: &[UiMountedAppearanceMechanic],
) -> Result<(), UiMountedAppearanceLoweringDenial> {
    for (ordinal, participant) in order.bottom_to_top().iter().enumerate() {
        let UiOverlayParticipantIdentity::Backdrop(identity) = participant else {
            continue;
        };
        let Some(UiMountedAppearanceMechanic::Backdrop(backdrop)) =
            mechanics.iter().find(|mechanic| {
                matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::Backdrop(candidate)
                        if candidate.identity() == identity
                )
            })
        else {
            return Err(UiMountedAppearanceLoweringDenial::OrderParticipantMissing);
        };
        let ordinal = u32::try_from(ordinal)
            .map_err(|_| UiMountedAppearanceLoweringDenial::WorkConstruction)?;
        if backdrop.placement().overlay_revision() != order.portal_revision()
            || backdrop.placement().ordinal() != ordinal
        {
            return Err(UiMountedAppearanceLoweringDenial::BackdropPlacementMismatch);
        }
    }
    Ok(())
}

fn lower_node(
    input: &UiMountedAppearanceLoweringInput,
    node: &UiMountedAppearanceNodeInput,
    mechanics: &mut Vec<UiMountedAppearanceMechanic>,
    records: &mut Vec<UiMountedAppearanceFact>,
) -> Result<(), UiMountedAppearanceLoweringDenial> {
    if node.semantic_surface != input.semantic_surface {
        return Err(UiMountedAppearanceLoweringDenial::NodeSurfaceMismatch);
    }
    if node.issuer.frame_identity() != input.frame
        || node.node_receipt
            != node
                .issuer
                .receipt_for(node.node_receipt.mounted_instance())
    {
        return Err(UiMountedAppearanceLoweringDenial::NodeReceiptFrameMismatch);
    }
    if !node.projection.matches_issuer(node.issuer) {
        return Err(UiMountedAppearanceLoweringDenial::NodeProjectionIssuerMismatch);
    }
    if node.portal_instance.is_some() && node.surface_paint.is_none() {
        return Err(UiMountedAppearanceLoweringDenial::PortalSurfaceMissing);
    }
    if let Some(portal_instance) = node.portal_instance {
        if portal_instance != node.node_receipt.mounted_instance() {
            return Err(UiMountedAppearanceLoweringDenial::PointerTargetMismatch);
        }
    }
    if let Some(pointer) = node.pointer {
        if pointer.target != node.node_receipt.mounted_instance() {
            return Err(UiMountedAppearanceLoweringDenial::PointerTargetMismatch);
        }
        if pointer.surface != input.semantic_surface {
            return Err(UiMountedAppearanceLoweringDenial::PointerSurfaceMismatch);
        }
    }
    if let Some(surface) = surface::lower(node)? {
        let damage = match &surface {
            UiMountedAppearanceMechanic::Surface(mechanic) => {
                UiMountedAppearanceVisualBounds::from_visual(mechanic.visual_bounds())
            }
            UiMountedAppearanceMechanic::PortalSurface(mechanic) => {
                UiMountedAppearanceVisualBounds::from_visual(mechanic.surface().visual_bounds())
            }
            _ => unreachable!("surface lowering returns a surface family"),
        };
        records.push(UiMountedAppearanceFact::node(
            input.semantic_surface,
            node.node_receipt,
            node.projection,
            node.semantic_digest,
            surface.clone(),
            super::fact::UiMountedAppearanceDamageShape::Visual {
                bounds: damage,
                attribution: worth_ui_host_contract::UiAppearanceDamageAttribution::Surface,
            },
        ));
        mechanics.push(surface);
    }
    if let Some(outline) = outline::lower(node)? {
        let damage = match &outline {
            UiMountedAppearanceMechanic::Outline(mechanic) => {
                UiMountedAppearanceVisualBounds::from_visual(mechanic.visual_bounds())
            }
            _ => unreachable!("outline lowering returns an outline family"),
        };
        records.push(UiMountedAppearanceFact::node(
            input.semantic_surface,
            node.node_receipt,
            node.projection,
            node.semantic_digest,
            outline.clone(),
            super::fact::UiMountedAppearanceDamageShape::Visual {
                bounds: damage,
                attribution: worth_ui_host_contract::UiAppearanceDamageAttribution::Outline,
            },
        ));
        mechanics.push(outline);
    }
    for text in text_foreground::lower(node)? {
        records.push(UiMountedAppearanceFact::node(
            input.semantic_surface,
            node.node_receipt,
            node.projection,
            node.semantic_digest,
            text.clone(),
            super::fact::UiMountedAppearanceDamageShape::Visual {
                bounds: UiMountedAppearanceVisualBounds::from_allocation(node.bounds),
                attribution: worth_ui_host_contract::UiAppearanceDamageAttribution::TextForeground,
            },
        ));
        mechanics.push(text);
    }
    if let Some(pointer) = pointer_affordance::lower(node)? {
        records.push(UiMountedAppearanceFact::node(
            input.semantic_surface,
            node.node_receipt,
            node.projection,
            node.semantic_digest,
            pointer.clone(),
            super::fact::UiMountedAppearanceDamageShape::None,
        ));
        mechanics.push(pointer);
    }
    Ok(())
}

fn require_issued_participants(
    order: &worth_ui_host_contract::UiMountedOverlayOrderMechanic,
    mechanics: &[UiMountedAppearanceMechanic],
) -> Result<(), UiMountedAppearanceLoweringDenial> {
    for participant in order.bottom_to_top() {
        let present = match participant {
            UiOverlayParticipantIdentity::Portal(instance) => mechanics.iter().any(|mechanic| {
                matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::PortalSurface(surface)
                        if surface.portal_instance() == *instance
                )
            }),
            UiOverlayParticipantIdentity::Backdrop(identity) => mechanics.iter().any(|mechanic| {
                matches!(
                    mechanic,
                    UiMountedAppearanceMechanic::Backdrop(backdrop)
                        if backdrop.identity() == identity
                )
            }),
        };
        if !present {
            return Err(UiMountedAppearanceLoweringDenial::OrderParticipantMissing);
        }
    }
    let ordered = order.bottom_to_top();
    let overlay_mechanics = mechanics.iter().filter_map(|mechanic| match mechanic {
        UiMountedAppearanceMechanic::PortalSurface(surface) => Some(
            UiOverlayParticipantIdentity::Portal(surface.portal_instance()),
        ),
        UiMountedAppearanceMechanic::Backdrop(backdrop) => Some(
            UiOverlayParticipantIdentity::Backdrop(backdrop.identity().clone()),
        ),
        _ => None,
    });
    for participant in overlay_mechanics {
        if !ordered.iter().any(|issued| issued == &participant) {
            return Err(UiMountedAppearanceLoweringDenial::OrderParticipantMissing);
        }
    }
    Ok(())
}
