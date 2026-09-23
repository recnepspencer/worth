use super::*;
use crate::native::presentation::appearance::mounted_mechanic_fixtures::{
    mounted_scroll_chrome, MountedScrollChromeFixtureInput,
};
use crate::native::presentation::{raster::UiNativeRasterBasis, UiNativeRasterOperation};
use worth_ui_host_contract::{
    UiMountedLogicalDamage, UiMountedPresentationSample, UiMountedPresentationSampleInput,
    UiMountedScrollChromePart,
};

#[test]
fn scroll_thumb_sample_crosses_native_preparation_and_rolls_back_without_moving_its_track() {
    let world = DrawListWorld::new();
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let mut retained = UiNativeRetainedDrawList::from_complete(
        frame,
        world.surface,
        world.binding,
        world.content,
        world.requirement.baseline(),
        &[],
        &[],
        UiMountedPaintOrderIntegrity::for_order(&[]),
        &[],
    )
    .unwrap();
    let mut appearance =
        UiNativeAppearanceRetained::new(UiNativeAppearanceScale::qualified(1_000).unwrap());
    let chrome = |part, height| {
        mounted_scroll_chrome(MountedScrollChromeFixtureInput {
            semantic_surface: world.surface,
            owner_instance: world.first,
            part,
            rect: UiAppearanceAllocationBounds::new(80_000, 10_000, 6_000, height).unwrap(),
            radius: 0,
            background: UiMountedAppearanceColor::from_straight_srgba([90, 90, 90, 255]),
        })
    };
    let track = chrome(UiMountedScrollChromePart::Track, 70_000);
    let thumb = chrome(UiMountedScrollChromePart::Thumb, 20_000);
    let track_key = appearance
        .insert(UiNativeAppearanceCommand::ScrollChrome(track), None)
        .unwrap();
    appearance
        .insert(
            UiNativeAppearanceCommand::ScrollChrome(thumb),
            Some(track_key),
        )
        .unwrap();
    retained.staged_appearance = Some((world.requirement, appearance));
    let bounds = |y, height| {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 80.0,
            y,
            width: 6.0,
            height,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap()
    };
    let identity = UiMountedPaintCommandIdentity::scroll_chrome(thumb.identity());
    assert_ne!(
        identity,
        UiMountedPaintCommandIdentity::appearance_surface(world.first)
    );
    assert_ne!(
        identity,
        UiMountedPaintCommandIdentity::scroll_chrome(track.identity())
    );
    let sample =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame,
            surface: world.surface,
            binding: world.binding,
            content: world.content,
            baseline: world.requirement.baseline(),
            production_cost: Default::default(),
            changes: vec![
                UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
                    identity,
                    UiMountedPresentationTransform::from_runtime_sampling(
                        bounds(10.0, 20.0),
                        bounds(40.0, 20.0),
                    )
                    .unwrap(),
                    thumb.opacity(),
                    bounds(10.0, 70.0),
                )
                .unwrap(),
            ],
            damage: vec![UiMountedLogicalDamage::from_runtime_mounting(bounds(
                10.0, 70.0,
            ))],
        })
        .unwrap();
    let basis = UiNativeRasterBasis::new([100, 100], 1.0);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .unwrap();
    let (plan, undo) = crate::native::presentation::sample::prepare_sample_plan(
        basis,
        &sample,
        &atlas,
        &mut retained,
    )
    .unwrap_or_else(|_| panic!("derived thumb sample crosses native physical preparation"));
    let rectangles = plan
        .operations
        .iter()
        .filter_map(|operation| match operation {
            UiNativeRasterOperation::Surface(surface) => Some(surface.rect().physical_bounds()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(rectangles.contains(&[80.0, 40.0, 6.0, 20.0]));
    assert!(rectangles.contains(&[80.0, 10.0, 6.0, 70.0]));
    assert!(
        retained.command(identity).is_none(),
        "chrome never acquires a node paint command"
    );
    assert_eq!(
        retained
            .sampled_chrome_observation(undo.changed_identities())
            .as_ref(),
        &[(thumb.identity(), [80_000, 40_000, 6_000, 20_000])]
    );
    retained.rollback_sample(undo).unwrap();
    assert!(retained.sample_override(identity).is_none());
    assert!(retained.sampled_chrome_observation([identity]).is_empty());
    let restored = retained
        .complete_appearance_operations(basis, &atlas)
        .unwrap();
    assert!(restored.iter().any(|operation| matches!(operation,
        UiNativeRasterOperation::Surface(surface) if surface.rect().physical_bounds() == [80.0, 10.0, 6.0, 20.0]
    )));
}
