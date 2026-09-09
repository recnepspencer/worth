use super::foreground_coverage_test_world::CoverageWorld;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPresentationAttemptIdentity};
use worth_ui_host_native::{
    UiNativeTextForegroundAtlasModel, UiNativeTextForegroundFinalizationDenial as Denial,
};

#[test]
fn real_raster_images_finalize_exact_clipped_coverage_and_reuse_atlas_hits() {
    let world = CoverageWorld::new(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [5.0, 0.0, 70.0, 48.0],
    );
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let (missing, report) = world.with_native(world.attempt, |view| {
        atlas
            .finalize_existing(&world.fragment, view, &world.foreground, [400, 48])
            .err()
    });
    assert_eq!(missing, Some(Denial::MissingAtlasEntry));
    assert_eq!(report.rasterized_glyphs(), 0);
    let (output, report) = world.with_native(world.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [400, 48],
            )
            .unwrap()
    });
    assert!(report.rasterized_glyphs() > 0);
    assert_eq!(
        output.regions().as_ref(),
        world.expected_images(0, [5, 0, 75, 48])
    );
    assert_eq!(output.cost().image_commands, 2);
    assert_eq!(atlas.validate_images(&output), Ok(()));
    let foreign_atlas = UiNativeTextForegroundAtlasModel::new();
    assert_eq!(
        foreign_atlas.validate_images(&output),
        Err(Denial::AtlasImageBasis)
    );
    let (reused, report) = world.with_native(world.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [400, 48],
            )
            .unwrap()
    });
    assert_eq!(reused.regions(), output.regions());
    assert_eq!(atlas.validate_images(&output), Err(Denial::AtlasImageBasis));
    assert_eq!(atlas.validate_images(&reused), Ok(()));
    // Historical coverage survives invalidation of its candidate image basis.
    assert_eq!(
        output.regions().as_ref(),
        world.expected_images(0, [5, 0, 75, 48])
    );
    assert_eq!(report.rasterized_glyphs(), 0);
    let (foreign, report) = world.with_native(
        UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        |view| {
            atlas
                .rasterize_with_simulated_submission(
                    &world.fragment,
                    view,
                    &world.foreground,
                    [400, 48],
                )
                .err()
        },
    );
    assert_eq!(foreign, Some(Denial::FragmentAffinity));
    assert_eq!(report.rasterized_glyphs(), 0);
}

#[test]
fn finalized_foreground_prepares_existing_glyph_vertex_paint() {
    let world = CoverageWorld::with_paint(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[(worth_ui_host_contract::UiSemanticTextSlot::Value, 0.0)],
        ([0, 255, 0, 128], 20_000),
    );
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let (output, _) = world.with_native(world.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [400, 48],
            )
            .unwrap()
    });
    assert_eq!(
        output.regions().as_ref(),
        world.expected_images(0, [0, 0, 400, 48])
    );
    let colors = output.glyph_vertex_colors([400, 48]);
    assert_eq!(colors.len(), 3, "one prepared command per admitted image");
    // Token foreground is white. Endpoint green independently checks decoded RGB;
    // non-endpoint alpha/opacity detects ignored, doubled, or requantized factors.
    let expected_alpha = (128.0_f64 / 255.0 * 20_000.0 / 65_535.0) as f32;
    for color in colors.iter() {
        assert_eq!(&color[..3], &[0.0, 1.0, 0.0]);
        assert!((color[3] - expected_alpha).abs() <= f32::EPSILON);
    }
    let (reused, report) = world.with_native(world.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [400, 48],
            )
            .unwrap()
    });
    assert_eq!(reused.glyph_vertex_colors([400, 48]), colors);
    assert_eq!(report.rasterized_glyphs(), 0);
}

#[test]
fn complete_empty_and_fully_clipped_glyph_images_are_distinct_from_missing_atlas() {
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let empty = CoverageWorld::new(
        " ",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 160.0, 48.0],
    );
    let (result, report) = empty.with_native(empty.attempt, |view| {
        atlas
            .finalize_existing(&empty.fragment, view, &empty.foreground, [400, 48])
            .unwrap()
    });
    assert!(result.regions().is_empty());
    assert_eq!(report.rasterized_glyphs(), 0);
    let clipped = CoverageWorld::new(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [160.0, 0.0, 10.0, 48.0],
    );
    let (missing, _) = clipped.with_native(clipped.attempt, |view| {
        atlas
            .finalize_existing(&clipped.fragment, view, &clipped.foreground, [400, 48])
            .err()
    });
    assert_eq!(missing, Some(Denial::MissingAtlasEntry));
    let (complete, _) = clipped.with_native(clipped.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &clipped.fragment,
                view,
                &clipped.foreground,
                [400, 48],
            )
            .unwrap()
    });
    assert!(complete.regions().is_empty());
}

#[test]
fn finalized_text_replacement_and_removal_keep_other_targets_and_disjoint_damage() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first = CoverageWorld::new("W\tW\tW", instance, 0.0, [0.0, 0.0, 400.0, 48.0]);
    let second = CoverageWorld::in_binding(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        200.0,
        [0.0, 0.0, 400.0, 48.0],
        Some(&first),
    );
    let next = CoverageWorld::in_binding(
        "W\tW\tW",
        instance,
        10.0,
        [0.0, 0.0, 400.0, 48.0],
        Some(&second),
    );
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let mut prepare = |world: &CoverageWorld| {
        world
            .with_native(world.attempt, |view| {
                atlas
                    .rasterize_with_simulated_submission(
                        &world.fragment,
                        view,
                        &world.foreground,
                        [400, 48],
                    )
                    .unwrap()
            })
            .0
    };
    let prepared_first = prepare(&first);
    let prepared_second = prepare(&second);
    let prepared_next = prepare(&next);
    let damage = atlas
        .certify_finalized_text_retention(
            prepared_first,
            prepared_second,
            prepared_next,
            [30, 0, 40, 48],
        )
        .unwrap();
    let expected = first
        .expected_images(0, [0, 0, 400, 48])
        .into_iter()
        .chain(next.expected_images(10, [0, 0, 400, 48]));
    let pixels = |rectangles: Vec<[i64; 4]>| {
        rectangles
            .into_iter()
            .flat_map(|r| (r[0]..r[2]).flat_map(move |x| (r[1]..r[3]).map(move |y| (x, y))))
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(pixels(damage.to_vec()), pixels(expected.collect()));
}

#[test]
fn one_adopted_span_finalizes_every_distinct_text_command() {
    use worth_ui_host_contract::UiSemanticTextSlot;
    let world = CoverageWorld::with_commands(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[
            (UiSemanticTextSlot::Value, 0.0),
            (UiSemanticTextSlot::Posture, 200.0),
        ],
    );
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let (output, report) = world.with_native(world.attempt, |view| {
        world
            .foregrounds
            .iter()
            .map(|foreground| {
                atlas
                    .rasterize_with_simulated_submission(
                        &world.fragment,
                        view,
                        foreground,
                        [400, 48],
                    )
                    .unwrap()
            })
            .collect::<Vec<_>>()
    });
    let expected = world
        .expected_images(0, [0, 0, 400, 48])
        .into_iter()
        .chain(world.expected_images(200, [0, 0, 400, 48]))
        .collect::<Vec<_>>();
    let regions = output
        .iter()
        .flat_map(|candidate| candidate.regions().into_vec())
        .collect::<Vec<_>>();
    assert_eq!(regions, expected);
    assert!(output.iter().all(|candidate| {
        candidate.cost().candidate_visits == 2 && candidate.cost().image_commands == 3
    }));
    assert!(report.rasterized_glyphs() > 0);
}

#[test]
fn text_coverage_uses_existing_delta_rejection_and_pending_rollback() {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let first = CoverageWorld::new("W\tW\tW", instance, 0.0, [0.0, 0.0, 400.0, 48.0]);
    let next = CoverageWorld::in_binding(
        "W\tW\tW",
        instance,
        10.0,
        [0.0, 0.0, 400.0, 48.0],
        Some(&first),
    );
    let mut model = UiNativeTextForegroundAtlasModel::new();
    first.with_native(first.attempt, |view| {
        let initial = model
            .rasterize_with_simulated_submission(
                &first.fragment,
                view,
                &first.foreground,
                [400, 48],
            )
            .unwrap();
        model
            .initialize_presentation_coverage(initial, view, [400, 48])
            .unwrap();
    });
    let twin = clipped_candidate_twin(&next);
    let (settled, report) = next.with_native(next.attempt, |view| {
        let candidate = model
            .rasterize_with_simulated_submission(&next.fragment, view, &next.foreground, [400, 48])
            .unwrap();
        assert_eq!(
            model.certify_presentation_coverage(candidate, &twin, view),
            Err(Denial::PresentationCoverage)
        );
        let candidate = model
            .rasterize_with_simulated_submission(&next.fragment, view, &next.foreground, [400, 48])
            .unwrap();
        model.certify_presentation_coverage(candidate, &next.fragment, view)
    });
    let damage = settled.unwrap();
    // Observe the returned presentation replay, not the finalizer's own bounds.
    let expected = first
        .expected_images(0, [0, 0, 400, 48])
        .into_iter()
        .chain(next.expected_images(10, [0, 0, 400, 48]));
    let pixels = |rectangles: Vec<[i64; 4]>| {
        rectangles
            .into_iter()
            .flat_map(|r| (r[0]..r[2]).flat_map(move |x| (r[1]..r[3]).map(move |y| (x, y))))
            .collect::<std::collections::BTreeSet<_>>()
    };
    assert_eq!(pixels(damage.to_vec()), pixels(expected.collect()));
    assert_eq!(report.rasterized_glyphs(), 0);
}

fn clipped_candidate_twin(
    world: &CoverageWorld,
) -> worth_ui_host_contract::UiUnpublishedAppearanceFragment {
    use worth_ui_host_contract::*;
    let original = &world.fragment.text_candidates()[0];
    let clip = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: 60.0,
        y: 0.0,
        width: 340.0,
        height: 48.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap();
    let candidate = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: original.content_generation(),
            frame: original.frame(),
            surface: original.surface(),
            binding: original.binding(),
            mounted_instance: original.mounted_instance(),
            node_receipt: original.node_receipt(),
            allocation_basis: original.allocation_basis(),
            bounds: original.bounds(),
            clip_bounds: clip,
            origin_x: original.origin_x(),
            origin_y: original.origin_y(),
            text: std::sync::Arc::from(original.text()),
            layout: world.layout.view(),
            slot: original.slot(),
            collection_row: original.collection_row().cloned(),
            foregrounds: std::sync::Arc::from(original.foregrounds()),
            profile: original.profile(),
            layer_semantic_order: original.layer_semantic_order(),
            capability_generation: original.capability_generation(),
            capability_profile_digest: original.capability_profile_digest(),
        },
    )
    .unwrap();
    assert_ne!(candidate.clip_bounds(), original.clip_bounds());
    UiUnpublishedAppearanceFragment::from_runtime_mounting(
        world.fragment.identity(),
        world.fragment.work().clone(),
        [candidate],
        world.fragment.surface_binding(),
        world.fragment.presentation_affinity(),
    )
    .unwrap()
}
