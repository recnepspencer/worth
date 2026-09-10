use super::*;
use crate::native::presentation::appearance::{
    UiNativeAppearanceRetained, UiNativeAppearanceScale,
};
use crate::native::presentation::retained_draw_list::tests::{command, DrawListWorld};
use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
    UiAppearanceNormalizedLogicalRadii, UiMountedAppearanceColor, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedNodeAppearanceAttribution,
    UiMountedNodeReceiptIssuer, UiMountedOverlayOrderMechanic, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationOpacity, UiMountedPresentationReconstruction,
    UiMountedPresentationReconstructionInput, UiMountedPresentationSampleChange,
    UiMountedPresentationTransform, UiMountedRgba8, UiMountedSurfaceAppearanceCompletionInput,
    UiMountedSurfaceAppearanceMechanic, UiMountedSurfaceBorderEdges, UiMountedSurfacePaint,
};

#[path = "render_order_tests/locality.rs"]
mod locality;

#[test]
fn complete_and_damage_join_surfaces_at_their_ordinary_text_slots() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let first = world.rect(frame, world.first, 0.0, UiMountedRgba8::new(1, 2, 3, 255));
    let (text, _, _) =
        crate::native::presentation::sample::tests::semantic_text::semantic_text(&world, frame);
    let third = world.rect_at_order(
        frame,
        world.third,
        40.0,
        UiMountedRgba8::new(4, 5, 6, 255),
        9,
    );
    let commands = [command(first), text, command(third)];
    let order = commands
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
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
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let third_key = appearance
        .insert(
            UiNativeAppearanceCommand::Surface(surface_at(third, 9)),
            None,
        )
        .unwrap();
    let first_key = appearance
        .insert(
            UiNativeAppearanceCommand::Surface(surface_at(first, 0)),
            Some(third_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [third_key, first_key, first_key, third_key],
        )
        .unwrap();
    assert_eq!(
        joined.len(),
        3,
        "static and sampled selection cannot paint twice"
    );
    assert!(matches!(joined[0], UiNativeRetainedRenderItem::Appearance(key) if key == first_key));
    assert!(
        matches!(joined[1], UiNativeRetainedRenderItem::Paint(id) if id == commands[1].identity())
    );
    assert!(matches!(joined[2], UiNativeRetainedRenderItem::Appearance(key) if key == third_key));
}

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
}

#[test]
fn issued_portal_anchor_does_not_require_optional_surface_paint() {
    use crate::native::presentation::appearance::mounted_mechanic_fixtures::backdrop_for_surface;
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let portal = crate::native::presentation::sample::tests::portal(&world, frame);
    let command = UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&portal),
        mechanic: portal,
    };
    let order = [UiMountedPaintOrderIdentity::for_command(command.identity())];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        std::slice::from_ref(&command),
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .unwrap();
    let backdrop = backdrop_for_surface(world.surface, 0);
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let backdrop_key = appearance
        .insert(UiNativeAppearanceCommand::Backdrop(backdrop.clone()), None)
        .unwrap();
    appearance
        .insert(
            UiNativeAppearanceCommand::OverlayOrder(
                UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
                    world.surface,
                    UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
                    1,
                    1,
                    [
                        UiOverlayParticipantIdentity::Backdrop(backdrop.identity().clone()),
                        UiOverlayParticipantIdentity::Portal(world.first),
                    ],
                )
                .unwrap(),
            ),
            Some(backdrop_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items([command.identity()], [backdrop_key])
        .unwrap();
    assert!(
        matches!(joined[0], UiNativeRetainedRenderItem::Appearance(key) if key == backdrop_key)
    );
    assert!(matches!(joined[1], UiNativeRetainedRenderItem::Paint(id) if id == command.identity()));
}

#[test]
fn backdrop_after_portal_stays_after_its_ordinary_content() {
    use crate::native::presentation::appearance::mounted_mechanic_fixtures::backdrop_for_surface;
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let portal = crate::native::presentation::sample::tests::portal(&world, frame);
    let portal = UiMountedPaintCommand::PortalOverlay {
        identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::portal_overlay(&portal),
        mechanic: portal,
    };
    let (child, _, _) =
        crate::native::presentation::sample::tests::semantic_text::semantic_text(&world, frame);
    let commands = [portal, child];
    let order = commands
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
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
    let backdrop = backdrop_for_surface(world.surface, 0);
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let backdrop_key = appearance
        .insert(UiNativeAppearanceCommand::Backdrop(backdrop.clone()), None)
        .unwrap();
    appearance
        .insert(
            UiNativeAppearanceCommand::OverlayOrder(
                UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
                    world.surface,
                    UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
                    1,
                    1,
                    [
                        UiOverlayParticipantIdentity::Portal(world.first),
                        UiOverlayParticipantIdentity::Backdrop(backdrop.identity().clone()),
                    ],
                )
                .unwrap(),
            ),
            Some(backdrop_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let joined = retained
        .ordered_render_items(
            commands.iter().map(UiMountedPaintCommand::identity),
            [backdrop_key],
        )
        .unwrap();
    assert!(matches!(
        joined.as_ref(),
        [
            UiNativeRetainedRenderItem::Paint(portal),
            UiNativeRetainedRenderItem::Paint(child),
            UiNativeRetainedRenderItem::Appearance(backdrop),
        ] if *portal == commands[0].identity()
            && *child == commands[1].identity()
            && *backdrop == backdrop_key
    ));
}

#[test]
fn accepted_reconstruction_motion_moves_appearance_surface_onscreen_once() {
    let world = DrawListWorld::new();
    let predecessor = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(
        frame,
        world.first,
        120.0,
        UiMountedRgba8::new(30, 60, 90, 255),
    );
    let complete = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let sampled = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 20.0,
        y: 0.0,
        width: rect.bounds().width(),
        height: rect.bounds().height(),
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    let sample = UiMountedPresentationSampleChange::from_runtime_sampling(
        identity,
        Some(
            UiMountedPresentationTransform::from_runtime_sampling(rect.bounds(), sampled).unwrap(),
        ),
        UiMountedPresentationOpacity::from_runtime_composition(32_768),
    );
    let reconstruction = UiMountedPresentationReconstruction::from_inert_mechanics(
        UiMountedPresentationReconstructionInput {
            predecessor,
            successor: frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            projection: complete.projection().clone(),
            commands: complete.commands().to_vec(),
            sample_overrides: vec![sample],
            order: complete.order().to_vec(),
            order_integrity: complete.order_integrity(),
            damage: complete.damage().to_vec(),
            production_cost: Default::default(),
        },
    );
    let mut retained = UiNativeRetainedDrawList::reconstruction(&reconstruction, &[]).unwrap();
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    appearance
        .insert(UiNativeAppearanceCommand::Surface(surface(rect)), None)
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));

    let operations = retained
        .complete_appearance_operations(
            crate::native::presentation::raster::UiNativeRasterBasis::new([100, 100], 1.0),
            &crate::native::text_atlas::UiNativeTextAtlas::new(),
        )
        .unwrap();
    assert!(matches!(
        operations.as_slice(),
        [crate::native::presentation::UiNativeRasterOperation::Surface(operation)]
            if operation.rect().physical_bounds() == [20.0, 0.0, 32.0, 24.0]
    ));
}

fn surface(
    source: worth_ui_host_contract::UiMountedFilledRectMechanic,
) -> UiMountedSurfaceAppearanceMechanic {
    surface_at(source, 0)
}

fn surface_at(
    source: worth_ui_host_contract::UiMountedFilledRectMechanic,
    surface_paint_order: u32,
) -> UiMountedSurfaceAppearanceMechanic {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(source.frame()).unwrap();
    let bounds = source.bounds();
    let allocation = UiAppearanceAllocationBounds::new(
        (bounds.x() * 1_000.0) as i32,
        (bounds.y() * 1_000.0) as i32,
        (bounds.width() * 1_000.0) as u32,
        (bounds.height() * 1_000.0) as u32,
    )
    .unwrap();
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: source.node_receipt(),
            bounds: allocation,
            clip: UiAppearanceClip::new(
                allocation.x(),
                allocation.y(),
                allocation.width(),
                allocation.height(),
            )
            .unwrap(),
            surface_paint_order,
            radii: UiAppearanceNormalizedLogicalRadii::normalize(
                allocation,
                [UiAppearanceLogicalLength::ZERO; 4],
            ),
            border_edges: UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint: UiMountedSurfacePaint::Fill(UiMountedAppearanceColor::from_straight_srgba([
                9, 10, 11, 255,
            ])),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}
