//! Retained real text images decide physical Motion work when logical damage is empty.
use super::image_oracle::{clear, images};
use super::{bounds, rectangle, CoverageWorld};
use worth_ui_host_contract::*;
use worth_ui_host_native::UiNativeTextReplayOperation as Op;

#[test]
fn zero_logical_damage_samples_replay_visible_overhang_and_accept_empty_images() {
    for source in ["W\tW\tW", "   "] {
        let world = CoverageWorld::with_physical_geometry(
            source,
            UiMountedInstanceIdentity::mint_unbound().unwrap(),
            0.0,
            [56.0, 0.0, 344.0, 48.0],
            None,
            &[(UiSemanticTextSlot::Value, 0.0)],
            ([0, 255, 0, 128], 20_000),
            (1.0, 1_250),
        );
        let text = world.fragment.text_candidates()[0].clone();
        assert!(text.bounds().x() + text.bounds().width() < text.clip_bounds().x());
        let command = UiMountedPaintCommand::SemanticText {
            identity: UiMountedPaintCommandIdentity::semantic_text(&text),
            mechanic: text.clone(),
        };
        let id = command.identity();
        let commands = [command, rectangle(&text, bounds([380.0, 0.0, 8.0, 8.0]))];
        let order = commands
            .iter()
            .map(|c| UiMountedPaintOrderIdentity::for_command(c.identity()))
            .collect::<Vec<_>>();
        let mut model = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
        world.with_native(world.attempt, |view| {
            model
                .rasterize_with_simulated_submission(
                    &world.fragment,
                    view,
                    &world.foreground,
                    [500, 60],
                )
                .unwrap();
            model
                .initialize_ordinary_presentation(&commands, &order, view, [500, 60])
                .unwrap();
        });
        let affinity = world.fragment.presentation_affinity();
        let sample =
            UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
                frame: text.frame(),
                surface: affinity.surface(),
                binding: affinity.binding(),
                content: affinity.content(),
                baseline: affinity.baseline(),
                changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
                    id,
                    None,
                    UiMountedPresentationOpacity::from_runtime_composition(32_768),
                )],
                damage: vec![],
                production_cost: Default::default(),
            });
        let sample = sample.unwrap();
        let (operations, cost) = model.certify_ordinary_sample_retry(&sample).unwrap();
        if source == "   " {
            assert!(
                operations.is_empty(),
                "authenticated empty images need no physical work"
            );
            assert_eq!(cost.presented_surfaces(), 0);
            assert_eq!(cost.presented_pixels(), 0);
            assert_eq!(cost.cleared_pixels(), 0);
            assert_eq!(cost.rendered_pixels(), 0);
            assert_eq!(cost.queue_submissions(), 0);
            assert_eq!(cost.presents(), 0);
        } else {
            let expected = images(&world)
                .into_iter()
                .filter(|image| image[0] >= 70.0)
                .collect::<Vec<_>>();
            assert_eq!(expected.len(), 2);
            assert_eq!(
                operations.len(),
                4,
                "two image clears and glyphs; distant paint untouched"
            );
            for (index, image) in expected.iter().enumerate() {
                assert_eq!(
                    operations[index * 2],
                    Op::Clear {
                        bounds: clear(*image)
                    }
                );
                let Op::Glyph {
                    run,
                    bounds,
                    vertex_color,
                } = operations[index * 2 + 1]
                else {
                    panic!("visible image replay");
                };
                assert_eq!(bounds, *image);
                assert_eq!(run.mechanic(), id);
                assert_eq!(vertex_color[..3], [1.0; 3]);
                assert!((vertex_color[3] - 32_768.0 / 65_535.0).abs() < 0.000001);
            }
            assert!(cost.cleared_pixels() > 0);
            assert_eq!(cost.presented_surfaces(), 1);
        }
    }
}

#[test]
fn accepted_motion_opacity_is_applied_once_to_adopted_glyph_vertices() {
    let world = CoverageWorld::with_paint(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[(UiSemanticTextSlot::Value, 0.0)],
        ([255, 255, 255, 255], 40_000),
    );
    let text = world.fragment.text_candidates()[0].clone();
    let command = UiMountedPaintCommandIdentity::semantic_text(&text);
    let mut model = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    world.with_native(world.attempt, |view| {
        let candidate = model
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [400, 48],
            )
            .unwrap();
        model
            .initialize_presentation_coverage(candidate, view, [400, 48])
            .unwrap();
    });
    let affinity = world.fragment.presentation_affinity();
    let sample =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame: text.frame(),
            surface: affinity.surface(),
            binding: affinity.binding(),
            content: affinity.content(),
            baseline: affinity.baseline(),
            changes: vec![UiMountedPresentationSampleChange::from_runtime_sampling(
                command,
                None,
                UiMountedPresentationOpacity::from_runtime_composition(20_000),
            )],
            damage: vec![],
            production_cost: Default::default(),
        })
        .unwrap();
    let (operations, _) = model.certify_ordinary_sample_retry(&sample).unwrap();
    let expected = 20_000.0 / 65_535.0;
    let glyphs = operations
        .iter()
        .filter_map(|operation| match operation {
            Op::Glyph { vertex_color, .. } => Some(vertex_color[3]),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!glyphs.is_empty());
    assert!(glyphs
        .iter()
        .all(|alpha| (*alpha - expected).abs() < 0.000_001));
}
