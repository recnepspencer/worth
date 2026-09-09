use crate::native::presentation::retained_raster::build_plan;
use crate::native::presentation::{
    raster::UiNativeRasterBasis,
    retained_draw_list::tests::{command, DrawListWorld},
    UiNativeRasterOperation, UiNativeRetainedDrawList,
};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedLogicalDamage,
    UiMountedNodeReceiptIssuer, UiMountedPaintCommand, UiMountedPaintCommandIdentity,
    UiMountedPaintOrderIdentity, UiMountedPaintOrderIntegrity, UiMountedPortalInputShielding,
    UiMountedPortalOverlayCompletionInput, UiMountedPortalOverlayLifecyclePosture,
    UiMountedPortalOverlayMechanic, UiMountedPresentationOpacity, UiMountedPresentationSample,
    UiMountedPresentationSampleChange, UiMountedPresentationSampleInput,
    UiMountedPresentationTransform, UiMountedRgba8,
};

#[path = "sample_tests/rendering.rs"]
mod rendering;
#[path = "sample_tests/semantic_text.rs"]
mod semantic_text;
use rendering::render_sample_pixels;

#[test]
fn sampled_rect_moves_and_applies_alpha_without_mutating_semantic_command() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(
        frame,
        world.first,
        10.0,
        UiMountedRgba8::new(30, 60, 90, 255),
    );
    let initial = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let semantic_command = command(rect);
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let sample = sample(&world, frame, identity, 10.0, 30.0, 32_768);

    let basis = UiNativeRasterBasis::new([100, 100], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();
    let plan = build_plan(
        basis,
        &retained,
        replay,
        0,
        &crate::native::text_atlas::UiNativeTextAtlas::new(),
    )
    .unwrap();

    assert_eq!(retained.frame(), frame);
    assert_eq!(retained.command(identity), Some(&semantic_command));
    assert!(plan.operations.iter().any(|operation| matches!(
        operation,
        UiNativeRasterOperation::FilledRect {
            rect,
            source_rgba8: [30, 60, 90, 128],
        } if rect.physical_bounds()[0] == 30.0
    )));
}

#[test]
fn rejected_successor_sample_restores_the_previous_override() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(frame, world.first, 10.0, UiMountedRgba8::new(1, 2, 3, 255));
    let initial = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let first = sample(&world, frame, identity, 10.0, 20.0, 49_151);
    retained.stage_sample(&first).unwrap();
    let successor = sample(&world, frame, identity, 10.0, 40.0, 16_384);
    let (_replay, undo) = retained.stage_sample(&successor).unwrap();

    retained.rollback_sample(undo).unwrap();

    let restored = retained.sample_override(identity).unwrap();
    assert!((restored.opacity().factor() - 0.75).abs() < 0.001);
    assert_eq!(restored.transform().unwrap().sampled().x(), 20.0);
}

#[test]
fn stale_or_coordinate_mismatched_sample_denies_without_override_mutation() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(frame, world.first, 10.0, UiMountedRgba8::new(1, 2, 3, 255));
    let identity = command(rect).identity();
    let mut retained =
        UiNativeRetainedDrawList::initial(&world.initial(frame, [rect]), &[]).unwrap();

    let foreign_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let stale = sample(&world, foreign_frame, identity, 10.0, 20.0, 32_768);
    assert!(retained.stage_sample(&stale).is_err());
    assert_eq!(retained.sample_override(identity), None);

    let wrong_space = sample_with_bounds(
        &world,
        frame,
        identity,
        viewport_box(10.0, 0.0, 10.0, 10.0),
        viewport_box(20.0, 0.0, 10.0, 10.0),
        32_768,
    );
    assert!(retained.stage_sample(&wrong_space).is_err());
    assert_eq!(retained.sample_override(identity), None);
    assert_eq!(retained.frame(), frame);
}

#[test]
fn offscreen_sample_commits_derived_state_without_native_paint_cost() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let rect = world.rect(frame, world.first, 120.0, UiMountedRgba8::new(1, 2, 3, 255));
    let initial = world.initial(frame, [rect]);
    let identity = command(rect).identity();
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    let sample = sample(&world, frame, identity, 120.0, 140.0, 32_768);

    let basis = UiNativeRasterBasis::new([100, 100], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();
    let plan = build_plan(
        basis,
        &retained,
        replay,
        0,
        &crate::native::text_atlas::UiNativeTextAtlas::new(),
    )
    .unwrap();

    assert_eq!(retained.frame(), frame);
    assert_eq!(
        retained.sample_override(identity),
        Some(sample.changes()[0])
    );
    assert!(plan.operations.is_empty());
    assert_eq!(plan.cost.presented_surfaces(), 0);
    assert_eq!(plan.cost.surface_acquisitions(), 0);
    assert_eq!(plan.cost.queue_submissions(), 0);
    assert_eq!(plan.cost.presents(), 0);
}

#[test]
fn production_sample_plan_moves_clipped_portal_and_renders_expected_pixels() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let portal = portal(&world, frame);
    let command = UiMountedPaintCommand::PortalOverlay {
        identity: UiMountedPaintCommandIdentity::portal_overlay(&portal),
        mechanic: portal,
    };
    let identity = command.identity();
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
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
    let source = viewport_box(10.0, 0.0, 30.0, 20.0);
    let sampled = viewport_box(40.0, 0.0, 30.0, 20.0);
    let sample = sample_with_bounds(&world, frame, identity, source, sampled, 32_768);
    let basis = UiNativeRasterBasis::new([80, 32], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (mut replay, mut undo) = retained.stage_sample(&sample).unwrap();
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .unwrap();
    let plan = build_plan(
        basis,
        &retained,
        replay,
        0,
        &crate::native::text_atlas::UiNativeTextAtlas::new(),
    )
    .unwrap();

    let filled = plan
        .operations
        .iter()
        .find_map(|operation| match operation {
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                Some((rect.physical_bounds(), *source_rgba8))
            }
            UiNativeRasterOperation::Clear(_) | UiNativeRasterOperation::Glyph(_) => None,
        });
    assert_eq!(filled, Some(([45.0, 0.0, 10.0, 20.0], [220, 40, 20, 128])));
    assert_eq!(retained.frame(), frame);
    assert_eq!(retained.command(identity), Some(&command));
    assert_eq!(portal.anchor_bounds(), viewport_box(4.0, 4.0, 4.0, 4.0));
    assert!(plan.cost.logical_damage_regions() >= 2);

    let pixels = render_sample_pixels(basis, portal, plan);
    assert_eq!(pixels[0], [0, 0, 0, 0]);
    assert!(
        pixels[1][0] > 150
            && pixels[1][0] > pixels[1][1].saturating_mul(4)
            && pixels[1][3].abs_diff(128) <= 1,
        "sampled clipped portal pixel is {:?}",
        pixels[1]
    );
    assert_eq!(pixels[2], [0, 0, 0, 0]);
}

fn sample(
    world: &DrawListWorld,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    identity: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    source_x: f32,
    sampled_x: f32,
    opacity_units: u16,
) -> UiMountedPresentationSample {
    let source = bounds(source_x);
    let sampled = bounds(sampled_x);
    sample_with_bounds(world, frame, identity, source, sampled, opacity_units)
}

fn sample_with_bounds(
    world: &DrawListWorld,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    identity: worth_ui_host_contract::UiMountedPaintCommandIdentity,
    source: UiMountedCanonicalBox,
    sampled: UiMountedCanonicalBox,
    opacity_units: u16,
) -> UiMountedPresentationSample {
    UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
        frame,
        surface: world.surface,
        binding: world.binding,
        content: world.content,
        baseline: world.requirement.baseline(),
        changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
            identity,
            Some(UiMountedPresentationTransform::from_runtime_sampling(source, sampled).unwrap()),
            UiMountedPresentationOpacity::from_runtime_composition(opacity_units),
        )],
        damage: vec![
            UiMountedLogicalDamage::from_runtime_mounting(source),
            UiMountedLogicalDamage::from_runtime_mounting(sampled),
        ],
        production_cost: Default::default(),
    })
    .unwrap()
}

fn portal(
    world: &DrawListWorld,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
) -> UiMountedPortalOverlayMechanic {
    UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
        UiMountedPortalOverlayCompletionInput {
            frame,
            surface: world.surface,
            binding: world.binding,
            owner: world.first,
            owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                .unwrap()
                .receipt_for(world.first),
            portal_identity: 7,
            anchor_presentation: UiHostObservationPresentationBasis::new(
                world.requirement.host_surface(),
                frame,
                world.binding,
                UiHostPresentationEpoch::issued_by_host(1),
            ),
            anchor_bounds: viewport_box(4.0, 4.0, 4.0, 4.0),
            bounds: viewport_box(10.0, 0.0, 30.0, 20.0),
            clip_bounds: viewport_box(15.0, 0.0, 10.0, 20.0),
            color: UiMountedRgba8::new(220, 40, 20, 255),
            layer_semantic_order: 1,
            layer_depth: 1,
            lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
            shielding: UiMountedPortalInputShielding::ContentBounds,
        },
    )
    .unwrap()
}

fn bounds(x: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y: 0.0,
        width: 10.0,
        height: 10.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}

fn viewport_box(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::Viewport,
    })
    .unwrap()
}
