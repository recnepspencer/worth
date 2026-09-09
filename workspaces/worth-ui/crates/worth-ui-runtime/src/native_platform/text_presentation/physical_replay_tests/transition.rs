//! Actual outline images through ordinary text movement, refusal/retry and removal.
use super::image_oracle::{clear, images, intersection};
use super::{bounds, rectangle, CoverageWorld};
use worth_ui_host_contract::*;
use worth_ui_host_native::UiNativeTextReplayOperation as Op;

#[test]
fn overhanging_text_transition_replays_old_and_new_images_after_refusal() {
    let first = world(0.0, None);
    let second = world(160.0, Some(&first));
    let old = text_command(&first);
    let new = text_command(&second);
    assert_ne!(
        old.identity(),
        new.identity(),
        "exercise both replacement identities"
    );
    let occluder = rectangle(
        &first.fragment.text_candidates()[0],
        bounds([112.0, 0.0, 180.0, 48.0]),
    );
    let distant = rectangle(
        &first.fragment.text_candidates()[0],
        bounds([380.0, 0.0, 8.0, 8.0]),
    );
    let commands = [old.clone(), occluder.clone(), distant.clone()];
    let old_order = commands
        .iter()
        .map(|c| UiMountedPaintOrderIdentity::for_command(c.identity()))
        .collect::<Vec<_>>();
    let new_order = [
        UiMountedPaintOrderIdentity::for_command(new.identity()),
        old_order[1],
        old_order[2],
    ];
    let old_images = images(&first);
    let new_images = images(&second);
    assert_eq!(old_images.len(), 3);
    assert_eq!(new_images.len(), 3);
    assert!(
        old_images[2][0] > 1.25,
        "image overhang lies outside the allocation"
    );
    let mut model = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    let (_, initial_report) = first.with_native(first.attempt, |view| {
        model
            .rasterize_with_simulated_submission(
                &first.fragment,
                view,
                &first.foreground,
                [500, 60],
            )
            .unwrap();
        model
            .initialize_ordinary_presentation(&commands, &old_order, view, [500, 60])
            .unwrap();
    });
    assert!(initial_report.rasterized_glyphs() > 0);
    let (moved, moved_report) = second.with_native(second.attempt, |view| {
        // Real successor demand and atlas admission; no glyph record is rebuilt from IDs.
        model
            .rasterize_with_simulated_submission(
                &second.fragment,
                view,
                &second.foreground,
                [500, 60],
            )
            .unwrap();
        let affinity = view.presentation_work().affinity();
        let delta =
            UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
                predecessor: affinity.predecessor().unwrap(),
                successor: affinity.successor(),
                surface: affinity.surface(),
                binding: affinity.binding(),
                content: affinity.content(),
                baseline: affinity.baseline(),
                changes: vec![UiMountedPaintCommandChange::replacement(
                    old.identity(),
                    new.clone(),
                )],
                nodes: vec![],
                order: vec![
                    UiMountedPaintOrderEdit::remove(old_order[0]),
                    UiMountedPaintOrderEdit::place_after(new_order[0], None),
                ],
                order_integrity: UiMountedPaintOrderIntegrity::for_order(&new_order),
                damage: vec![
                    UiMountedLogicalDamage::from_runtime_mounting(old.bounds()),
                    UiMountedLogicalDamage::from_runtime_mounting(new.bounds()),
                ],
                auxiliary: None,
                production_cost: Default::default(),
            });
        model
            .certify_ordinary_delta_retry(&delta, view.text_raster_work().unwrap().glyph_runs())
            .unwrap()
    });
    assert_eq!(
        moved_report.rasterized_glyphs(),
        0,
        "integral physical translation reuses actual atlas images"
    );
    let mut expected = vec![[0.0, 0.0, 2.0, 60.0], [200.0, 0.0, 2.0, 60.0]];
    expected.extend(old_images.iter().chain(&new_images).copied().map(clear));
    assert_replay(&moved, &expected, &new_images, new.identity());

    let removal =
        UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
            predecessor: second.fragment.work().successor().frame(),
            successor: UiMountedFrameIdentity::mint_unbound().unwrap(),
            surface: second.fragment.text_candidates()[0].surface(),
            binding: second.fragment.text_candidates()[0].binding(),
            content: second.fragment.text_candidates()[0].content_generation(),
            baseline: first.fragment.surface_binding().baseline(),
            changes: vec![UiMountedPaintCommandChange::Remove(new.identity())],
            nodes: vec![],
            order: vec![UiMountedPaintOrderEdit::remove(new_order[0])],
            order_integrity: UiMountedPaintOrderIntegrity::for_order(&new_order[1..]),
            damage: vec![UiMountedLogicalDamage::from_runtime_mounting(new.bounds())],
            auxiliary: None,
            production_cost: Default::default(),
        });
    let removed = model.certify_ordinary_delta_retry(&removal, &[]).unwrap();
    let mut expected = vec![[200.0, 0.0, 2.0, 60.0]];
    expected.extend(new_images.iter().copied().map(clear));
    assert_replay(&removed, &expected, &[], new.identity());
}

fn world(x: f32, previous: Option<&CoverageWorld>) -> CoverageWorld {
    CoverageWorld::with_physical_geometry(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        x,
        [0.0, 0.0, 400.0, 48.0],
        previous,
        &[(UiSemanticTextSlot::Value, 0.0)],
        ([0, 255, 0, 128], 20_000),
        (1.0, 1_250),
    )
}

fn text_command(world: &CoverageWorld) -> UiMountedPaintCommand {
    let mechanic = world.fragment.text_candidates()[0].clone();
    UiMountedPaintCommand::SemanticText {
        identity: UiMountedPaintCommandIdentity::semantic_text(&mechanic),
        mechanic,
    }
}

fn assert_replay(
    ops: &[Op],
    expected: &[[f32; 4]],
    images: &[[f32; 4]],
    id: UiMountedPaintCommandIdentity,
) {
    let clears = ops
        .iter()
        .filter_map(|op| {
            if let Op::Clear { bounds } = op {
                Some(*bounds)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    // Changed membership is a set; compare exact rectangles without imposing hash iteration order.
    let mut actual = clears.clone();
    let mut expected = expected.to_vec();
    actual.sort_by(|a, b| a.partial_cmp(b).unwrap());
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(
        actual, expected,
        "no allocation substitution or hull across disjoint images"
    );
    let mut cursor = 0;
    for damage in clears {
        assert_eq!(ops[cursor], Op::Clear { bounds: damage });
        cursor += 1;
        for image in images {
            if let Some(bounds) = intersection(*image, damage) {
                let Op::Glyph {
                    run,
                    bounds: actual,
                    vertex_color,
                } = ops[cursor]
                else {
                    panic!("text must precede its occluder");
                };
                assert_eq!(actual, bounds);
                assert_eq!(run.mechanic(), id);
                assert_eq!(run.foreground().channels(), [255; 4]);
                assert_eq!(vertex_color, [1.0; 4]);
                cursor += 1;
            }
        }
        if let Some(bounds) = intersection([140.0, 0.0, 225.0, 60.0], damage) {
            assert_eq!(
                ops[cursor],
                Op::Solid {
                    bounds,
                    color: [30, 60, 90, 255]
                }
            );
            cursor += 1;
        }
    }
    assert_eq!(cursor, ops.len(), "distant retained paint receives no work");
}
