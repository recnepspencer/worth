use super::*;

#[test]
fn appearance_only_surfaces_use_semantic_order_without_legacy_rect_anchors() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let (text, _, _) =
        crate::native::presentation::sample::tests::semantic_text::semantic_text(&world, frame);
    let order = [UiMountedPaintOrderIdentity::for_command(text.identity())];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        std::slice::from_ref(&text),
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .unwrap();
    let before = surface_at(
        world.rect(frame, world.first, 0.0, UiMountedRgba8::new(1, 2, 3, 255)),
        4,
    );
    let after = surface_at(
        world.rect(frame, world.third, 40.0, UiMountedRgba8::new(4, 5, 6, 255)),
        12,
    );
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let before_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(before), None)
        .unwrap();
    let after_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(after), Some(before_key))
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items([text.identity()], [after_key, before_key])
        .unwrap();
    assert!(matches!(
        joined.as_ref(),
        [
            UiNativeRetainedRenderItem::Appearance(before),
            UiNativeRetainedRenderItem::Paint(text_identity),
            UiNativeRetainedRenderItem::Appearance(after),
        ] if *before == before_key && *text_identity == text.identity() && *after == after_key
    ));
    let (ordinal, attribution) = retained.top_paint_attribution().unwrap();
    assert_eq!(ordinal, 2);
    assert_eq!(attribution.mounted_instance, world.third);
    assert_eq!(attribution.color.channels(), [9, 10, 11, 255]);
}

#[test]
fn appearance_only_surface_outline_pairs_keep_canonical_semantic_order() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let commands = [];
    let order = [];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &commands,
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .unwrap();
    let (low, low_outline) = surface_outline_pair(
        world.rect(frame, world.first, 0.0, UiMountedRgba8::new(1, 2, 3, 255)),
        4,
    );
    let (high, high_outline) = surface_outline_pair(
        world.rect(frame, world.third, 40.0, UiMountedRgba8::new(4, 5, 6, 255)),
        12,
    );
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let high_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(high), None)
        .unwrap();
    let high_outline_key = appearance
        .insert(
            UiNativeAppearanceCommand::Outline(high_outline),
            Some(high_key),
        )
        .unwrap();
    let low_key = appearance
        .insert(UiNativeAppearanceCommand::Surface(low), None)
        .unwrap();
    let low_outline_key = appearance
        .insert(
            UiNativeAppearanceCommand::Outline(low_outline),
            Some(low_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items([], [high_outline_key, high_key, low_outline_key, low_key])
        .unwrap();
    assert!(matches!(
        joined.as_ref(),
        [
            UiNativeRetainedRenderItem::Appearance(first),
            UiNativeRetainedRenderItem::Appearance(second),
            UiNativeRetainedRenderItem::Appearance(third),
            UiNativeRetainedRenderItem::Appearance(fourth),
        ] if *first == low_key
            && *second == low_outline_key
            && *third == high_key
            && *fourth == high_outline_key
    ));
    let (ordinal, attribution) = retained.top_paint_attribution().unwrap();
    assert_eq!(ordinal, 3);
    assert_eq!(attribution.mounted_instance, world.third);
    assert_eq!(attribution.color.channels(), [20, 21, 22, 255]);
    retained.retain_current_paint_attribution();
    let appearance = &mut retained.staged_appearance.as_mut().unwrap().1;
    let outline_undo = appearance.stage_command_remove(high_outline_key).unwrap();
    let surface_undo = appearance.stage_command_remove(high_key).unwrap();
    assert_eq!(
        retained.top_paint_attribution().unwrap().1.mounted_instance,
        world.first
    );
    let appearance = &mut retained.staged_appearance.as_mut().unwrap().1;
    appearance.rollback_text(surface_undo).unwrap();
    appearance.rollback_text(outline_undo).unwrap();
    assert_eq!(
        retained.top_paint_attribution(),
        Some((ordinal, attribution))
    );
    let appearance = &mut retained.staged_appearance.as_mut().unwrap().1;
    for key in [high_outline_key, high_key, low_outline_key, low_key] {
        appearance.stage_command_remove(key).unwrap();
    }
    assert_eq!(
        retained.top_paint_attribution(),
        Some((ordinal, attribution)),
        "terminal removal preserves the exact last attributed paint"
    );
}

fn surface_outline_pair(
    source: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    surface_paint_order: u32,
) -> (
    UiMountedSurfaceAppearanceMechanic,
    UiMountedOutlineAppearanceMechanic,
) {
    surface_outline_pair_in_group(source, surface_paint_order, None)
}

pub(super) fn surface_outline_pair_in_group(
    source: worth_ui_host_contract::UiMountedPortalOverlayMechanic,
    surface_paint_order: u32,
    portal_group: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
) -> (
    UiMountedSurfaceAppearanceMechanic,
    UiMountedOutlineAppearanceMechanic,
) {
    let frame = source.frame();
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let bounds = source.bounds();
    let allocation = UiAppearanceAllocationBounds::new(
        (bounds.x() * 1_000.0) as i32,
        (bounds.y() * 1_000.0) as i32,
        (bounds.width() * 1_000.0) as u32,
        (bounds.height() * 1_000.0) as u32,
    )
    .unwrap();
    let radii = UiAppearanceNormalizedLogicalRadii::normalize(
        allocation,
        [UiAppearanceLogicalLength::ZERO; 4],
    );
    let clip = UiAppearanceClip::new(
        allocation.x(),
        allocation.y(),
        allocation.width(),
        allocation.height(),
    )
    .unwrap();
    let projection =
        UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1).unwrap();
    let node_receipt = issuer.receipt_for(source.owner());
    let surface = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt,
            bounds: allocation,
            clip,
            surface_paint_order,
            portal_group,
            radii,
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                9, 10, 11, 255,
            ])),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection,
        },
    )
    .unwrap();
    let geometry = UiAppearanceOutlineGeometry::admit(
        allocation,
        radii,
        UiAppearanceLogicalLength::new(1_000).unwrap(),
        UiAppearanceLogicalLength::ZERO,
        UiAppearanceLogicalLength::ZERO,
    )
    .unwrap();
    let outline = UiMountedOutlineAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedOutlineAppearanceCompletionInput {
            issuer,
            node_receipt,
            clip,
            surface_paint_order,
            portal_group,
            geometry,
            color: UiMountedAppearanceColor::from_straight_srgba([20, 21, 22, 255]),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap();
    (surface, outline)
}
