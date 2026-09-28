use super::*;
use crate::native::presentation::raster::UiNativeRasterBasis;
use crate::native::presentation::retained_draw_list::UiNativeRetainedReplayPlan;
use crate::native::presentation::retained_raster::build_plan;
use crate::native::presentation::UiNativeRasterOperation;

fn empty_replay() -> UiNativeRetainedReplayPlan {
    UiNativeRetainedReplayPlan {
        physical_text_regions: Vec::new(),
        baseline_rgba8: [0; 4],
        regions: Box::new([]),
        staged_appearance_regions: Box::new([]),
        counters: Default::default(),
        identity_overlay_effect: false,
    }
}

fn painted_list(world: &DrawListWorld, basis: UiNativeRasterBasis) -> UiNativeRetainedDrawList {
    let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
    let initial = world.initial(
        frame,
        [world.rect(frame, world.first, 10.0, UiMountedRgba8::new(1, 2, 3, 255))],
    );
    let mut retained = UiNativeRetainedDrawList::initial(&initial, &[]).unwrap();
    retained
        .initialize_physical_coverage(basis, &crate::native::text_atlas::UiNativeTextAtlas::new())
        .unwrap();
    retained.paint_target(1);
    retained
}

/// A resize hands the list a successor target that starts empty. The list
/// keeps its commands, clips its coverage to the new extent and repaints the
/// whole target, however little the frame itself changed, until a
/// presentation into that target is submitted.
#[test]
fn a_successor_target_is_repainted_whole_until_a_presentation_is_submitted() {
    let world = DrawListWorld::new();
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    let mut retained = painted_list(&world, UiNativeRasterBasis::new([100, 100], 1.25));
    retained
        .prepare_target(UiNativeRasterBasis::new([100, 100], 1.25), 1)
        .unwrap();
    assert!(!retained.repaint_owed(), "the painted target owes nothing");

    let successor = UiNativeRasterBasis::new([60, 80], 1.25);
    assert!(retained.owes_repaint(2));
    for _retry in 0..2 {
        // A refused or superseded attempt prepares the same target again.
        retained.prepare_target(successor, 2).unwrap();
        assert!(retained.repaint_owed());
        let plan = build_plan(successor, &mut retained, empty_replay(), 0, &atlas).unwrap();
        assert!(matches!(
            plan.operations.first(),
            Some(UiNativeRasterOperation::Clear(rect))
                if rect.physical_bounds() == [0.0, 0.0, 60.0, 80.0]
        ));
        let filled = plan
            .operations
            .iter()
            .filter_map(|operation| match operation {
                UiNativeRasterOperation::FilledRect { rect, source_rgba8 } => {
                    Some((rect.physical_bounds(), *source_rgba8))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        // 10..42 points at 1.25 is 12.5..52.5 device pixels, drawn from whole pixels.
        assert_eq!(
            filled,
            [([12.0, 0.0, 41.0, 30.0], [1, 2, 3, 255])],
            "retained paint replays into the successor target"
        );
    }

    retained.settle_target();
    assert!(!retained.owes_repaint(2));
    retained.prepare_target(successor, 2).unwrap();
    let plan = build_plan(successor, &mut retained, empty_replay(), 0, &atlas).unwrap();
    assert!(
        plan.operations.is_empty(),
        "a repainted target owes nothing"
    );
}

/// Physical coverage is measured in device pixels: a new scale cannot carry
/// onto a successor target and leaves the list to reconstruction.
#[test]
fn a_scale_change_refuses_the_successor_target() {
    let world = DrawListWorld::new();
    let mut retained = painted_list(&world, UiNativeRasterBasis::new([100, 100], 1.25));
    assert!(matches!(
        retained.prepare_target(UiNativeRasterBasis::new([100, 100], 1.5), 2),
        Err(crate::native::presentation::retained_draw_list::UiNativeRetainedDrawListDenial::AffinityMismatch)
    ));
}
