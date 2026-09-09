//! Empty adopted text preserves command validation and presentation rollback.
use super::foreground_coverage_test_world::CoverageWorld;
use worth_ui_host_contract::*;

#[test]
fn fully_clipped_adoption_still_validates_ordinary_command_before_empty_plan() {
    let world = CoverageWorld::new(
        "W",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 30.0, 400.0, 1.0],
    );
    let text = world.fragment.text_candidates()[0].clone();
    let identity = UiMountedPaintCommandIdentity::semantic_text(&text);
    let commands = [UiMountedPaintCommand::SemanticText {
        identity,
        mechanic: text.clone(),
    }];
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut atlas = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    world.with_native(world.attempt, |view| {
        let candidate = atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [500, 60],
            )
            .unwrap();
        assert!(
            candidate.regions().is_empty(),
            "no glyph image reaches the ancestor clip"
        );
        assert!(atlas
            .foreground_replay_commands(&commands, &order, view, [500, 60], &candidate)
            .unwrap()
            .is_empty());
        let narrower = text
            .clipped_to_appearance_ancestor(
                UiAppearanceClip::new(0, 30_000, 200_000, 1_000).unwrap(),
            )
            .unwrap()
            .unwrap();
        let mismatched = [UiMountedPaintCommand::SemanticText {
            identity,
            mechanic: narrower,
        }];
        assert!(
            atlas
                .foreground_replay_commands(&mismatched, &order, view, [500, 60], &candidate)
                .is_err(),
            "zero visible glyphs cannot bypass command correspondence"
        );
    });
}

#[test]
fn fully_clipped_successor_validates_after_staging_and_preserves_rejection_retry() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first = CoverageWorld::new("W\tW\tW", instance, 0.0, [0.0, 0.0, 400.0, 48.0]);
    let next = CoverageWorld::in_binding(
        "W\tW\tW",
        instance,
        0.0,
        [0.0, 30.0, 400.0, 1.0],
        Some(&first),
    );
    let mut atlas = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    let (old_regions, _) = first.with_native(first.attempt, |view| {
        let candidate = atlas
            .rasterize_with_simulated_submission(
                &first.fragment,
                view,
                &first.foreground,
                [500, 60],
            )
            .unwrap();
        let regions = candidate.regions();
        assert_eq!(regions.len(), 3);
        atlas
            .initialize_presentation_coverage(candidate, view, [500, 60])
            .unwrap();
        regions
    });
    let (damage, report) = next.with_native(next.attempt, |view| {
        let candidate = atlas
            .rasterize_with_simulated_submission(&next.fragment, view, &next.foreground, [500, 60])
            .unwrap();
        assert!(candidate.regions().is_empty());
        atlas
            .certify_presentation_coverage(candidate, &next.fragment, view)
            .unwrap()
    });
    assert_eq!(
        damage, old_regions,
        "only accepted predecessor images need clearing"
    );
    assert_eq!(report.rasterized_glyphs(), 0);
}
