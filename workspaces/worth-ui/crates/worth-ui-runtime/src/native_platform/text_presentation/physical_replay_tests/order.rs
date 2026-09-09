//! Order-only delta damage must include actual text overhang without new glyph work.
use super::image_oracle::{clear, images, intersection};
use super::{bounds, rectangle, CoverageWorld};
use worth_ui_host_contract::*;
use worth_ui_host_native::UiNativeTextReplayOperation as Op;

#[test]
fn text_order_only_replays_overhang_above_and_below_retained_occluder() {
    let world = CoverageWorld::with_physical_geometry(
        "W\tW\tW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[(UiSemanticTextSlot::Value, 0.0)],
        ([0, 255, 0, 128], 20_000),
        (1.0, 1_250),
    );
    let text = world.fragment.text_candidates()[0].clone();
    let text_command = UiMountedPaintCommand::SemanticText {
        identity: UiMountedPaintCommandIdentity::semantic_text(&text),
        mechanic: text.clone(),
    };
    let occluder = rectangle(&text, bounds([112.0, 0.0, 24.0, 48.0]));
    let distant = rectangle(&text, bounds([380.0, 0.0, 8.0, 8.0]));
    let commands = [text_command, occluder, distant];
    let order = commands
        .iter()
        .map(|c| UiMountedPaintOrderIdentity::for_command(c.identity()))
        .collect::<Vec<_>>();
    let expected_images = images(&world);
    assert_eq!(expected_images.len(), 3);
    assert!(expected_images[2][0] > text.bounds().width() * 1.25);
    let mut model = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    let (_, report) = world.with_native(world.attempt, |view| {
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
    assert!(report.rasterized_glyphs() > 0);
    let raised_frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let raised_order = [order[1], order[0], order[2]];
    let raised = order_delta(
        &world,
        text.frame(),
        raised_frame,
        UiMountedPaintOrderEdit::place_after(order[0], Some(order[1])),
        &raised_order,
    );
    let above = model.certify_ordinary_delta_retry(&raised, &[]).unwrap();
    assert_ordered_images(
        &above,
        &expected_images,
        commands[0].identity(),
        TextStack::Above,
    );

    let lowered = order_delta(
        &world,
        raised_frame,
        UiMountedFrameIdentity::mint_unbound().unwrap(),
        UiMountedPaintOrderEdit::place_after(order[0], None),
        &order,
    );
    let below = model.certify_ordinary_delta_retry(&lowered, &[]).unwrap();
    assert_ordered_images(
        &below,
        &expected_images,
        commands[0].identity(),
        TextStack::Below,
    );
}

fn order_delta(
    world: &CoverageWorld,
    predecessor: UiMountedFrameIdentity,
    successor: UiMountedFrameIdentity,
    edit: UiMountedPaintOrderEdit,
    order: &[UiMountedPaintOrderIdentity],
) -> UiMountedPresentationDelta {
    let affinity = world.fragment.presentation_affinity();
    UiMountedPresentationDelta::from_inert_mechanics(UiMountedPresentationDeltaInput {
        predecessor,
        successor,
        surface: affinity.surface(),
        binding: affinity.binding(),
        content: affinity.content(),
        baseline: affinity.baseline(),
        changes: vec![],
        nodes: vec![],
        order: vec![edit],
        order_integrity: UiMountedPaintOrderIntegrity::for_order(order),
        damage: vec![UiMountedLogicalDamage::from_runtime_mounting(
            world.fragment.text_candidates()[0].bounds(),
        )],
        auxiliary: None,
        production_cost: Default::default(),
    })
}

enum TextStack {
    Above,
    Below,
}

fn assert_ordered_images(
    ops: &[Op],
    images: &[[f32; 4]],
    id: UiMountedPaintCommandIdentity,
    stack: TextStack,
) {
    let expected = std::iter::once([0.0, 0.0, 2.0, 60.0])
        .chain(images.iter().copied().map(clear))
        .collect::<Vec<_>>();
    let actual = ops
        .iter()
        .filter_map(|op| match op {
            Op::Clear { bounds } => Some(*bounds),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual, expected,
        "order-only damage includes each image outside allocation"
    );
    let mut cursor = 0;
    for damage in expected {
        assert_eq!(ops[cursor], Op::Clear { bounds: damage });
        cursor += 1;
        let solid = intersection([140.0, 0.0, 30.0, 60.0], damage).map(|bounds| Op::Solid {
            bounds,
            color: [30, 60, 90, 255],
        });
        if let (TextStack::Above, Some(solid)) = (&stack, &solid) {
            assert_eq!(&ops[cursor], solid);
            cursor += 1;
        }
        for image in images {
            if let Some(bounds) = intersection(*image, damage) {
                let Op::Glyph {
                    run,
                    bounds: actual,
                    vertex_color,
                } = ops[cursor]
                else {
                    panic!("text follows declared order");
                };
                assert_eq!(actual, bounds);
                assert_eq!(run.mechanic(), id);
                assert_eq!(run.foreground().channels(), [255; 4]);
                assert_eq!(vertex_color, [1.0; 4]);
                cursor += 1;
            }
        }
        if let (TextStack::Below, Some(solid)) = (&stack, &solid) {
            assert_eq!(&ops[cursor], solid);
            cursor += 1;
        }
    }
    assert_eq!(cursor, ops.len(), "distant paint receives no replay");
}
