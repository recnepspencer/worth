//! Actual qualified glyph images, ordinary retained order, and fractional physical replay.
mod image_oracle;
mod motion;
mod order;
mod transition;

use super::foreground_coverage_test_world::CoverageWorld;
use worth_ui_host_contract::*;

#[test]
fn physical_replay_includes_unchanged_overhanging_text_before_its_occluder() {
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
    // Independently locate the third actual raster image; allocation is deliberately
    // narrow while the mounted ancestor clip admits all three tab-separated images.
    let demand = worth_ui_text::derive_glyph_raster_demand(
        &world.layout,
        worth_ui_text::UiGlyphRasterDemandRequest {
            paint_spans: text.foregrounds(),
            selection: worth_ui_text::UiGlyphRasterDemandSelection::CompleteLayout,
            scale: worth_ui_text::UiGlyphRasterScale::new(1_250, text.qualified_layout_scale())
                .unwrap(),
            placement: worth_ui_text::UiGlyphRasterPlacement::from_mounted_logical(0.0, 0.0)
                .unwrap(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap();
    let raster = worth_ui_text::rasterize_alpha_outline(&world.layout, &demand).unwrap();
    let (ordinal, third) = demand
        .records()
        .iter()
        .enumerate()
        .find(|(_, record)| {
            record.attribution().original_range() == UiTextOriginalRange::new(4, 5).unwrap()
        })
        .unwrap();
    let positioned = demand
        .positioned_glyph_for_record(&world.layout, ordinal)
        .unwrap();
    let image = raster
        .batch()
        .records()
        .iter()
        .find(|image| image.key() == third.key())
        .unwrap();
    let left = (positioned.origin_x_millipoints() as f64 * 1.25 / 1_000.0).floor()
        + f64::from(image.bearing().x_over_64()) / 64.0;
    let top = (positioned.origin_y_millipoints() as f64 * 1.25 / 1_000.0).floor()
        - f64::from(image.bearing().y_over_64()) / 64.0;
    assert!(image.extent().width() > 2 && image.extent().height() > 2);
    assert!(left > f64::from(text.bounds().x() + text.bounds().width()) * 1.25);
    let damage_bounds = bounds([
        (left + 1.0) as f32 / 1.25,
        (top + 1.0) as f32 / 1.25,
        0.8,
        0.8,
    ]);
    let text_command = UiMountedPaintCommand::SemanticText {
        identity: UiMountedPaintCommandIdentity::semantic_text(&text),
        mechanic: text.clone(),
    };
    let occluder = rectangle(&text, damage_bounds);
    let distant = rectangle(&text, bounds([300.0, 0.0, 8.0, 8.0]));
    let expected = [text_command.identity(), occluder.identity()];
    let commands = [text_command, occluder, distant];
    let order = commands
        .iter()
        .map(|command| UiMountedPaintOrderIdentity::for_command(command.identity()))
        .collect::<Vec<_>>();
    let mut atlas = worth_ui_host_native::UiNativeTextForegroundAtlasModel::new();
    let (((selected, draws), candidate), report) = world.with_native(world.attempt, |view| {
        let candidate = atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [500, 60],
            )
            .unwrap();
        let replay = atlas
            .physical_replay_commands(
                &commands,
                &order,
                view,
                [500, 60],
                UiMountedLogicalDamage::from_runtime_mounting(damage_bounds),
            )
            .unwrap();
        (replay, candidate)
    });
    assert!(
        report.rasterized_glyphs() > 0,
        "the oracle reaches actual raster admission"
    );
    assert_eq!(selected.as_ref(), expected);
    assert_eq!(
        draws.as_ref(),
        [Some(expected[0]), None],
        "unchanged text is redrawn once below the occluder"
    );
    let ((operations, reconstructed), paint_report) = world.with_native(world.attempt, |view| {
        let operations = atlas
            .foreground_replay_commands(&commands, &order, view, [500, 60], &candidate)
            .unwrap();
        let changed = text
            .clone()
            .clipped_to_appearance_ancestor(UiAppearanceClip::new(0, 0, 50_000, 48_000).unwrap())
            .unwrap()
            .unwrap();
        let mut mismatched = commands.clone();
        mismatched[0] = UiMountedPaintCommand::SemanticText {
            identity: UiMountedPaintCommandIdentity::semantic_text(&changed),
            mechanic: changed,
        };
        assert!(atlas
            .foreground_replay_commands(&mismatched, &order, view, [500, 60], &candidate)
            .is_err());
        assert!(atlas
            .foreground_reconstruction_commands(&mismatched, &order, view, [500, 60], &candidate)
            .is_err());
        let reconstructed = atlas
            .foreground_reconstruction_commands(&commands, &order, view, [500, 60], &candidate)
            .unwrap();
        (operations, reconstructed)
    });
    assert_eq!(paint_report.rasterized_glyphs(), 0);
    use worth_ui_host_native::UiNativeTextReplayOperation as Operation;
    assert_eq!(
        reconstructed.len(),
        6,
        "full clear, three text images, occluder and distant rectangle"
    );
    assert!(matches!(
        reconstructed[0],
        Operation::Clear {
            bounds: [0.0, 0.0, 500.0, 60.0]
        }
    ));
    fn glyphs(ops: &[Operation]) -> Vec<&Operation> {
        ops.iter()
            .filter(|op| matches!(op, Operation::Glyph { .. }))
            .collect()
    }
    assert_eq!(
        glyphs(&reconstructed),
        glyphs(&operations),
        "reconstruction preserves exact adopted images and paint"
    );
    let expected_clears = demand
        .records()
        .iter()
        .enumerate()
        .filter_map(|(ordinal, record)| {
            let original = record.attribution().original_range();
            if ![0, 2, 4].contains(&original.start()) {
                return None;
            }
            let positioned = demand
                .positioned_glyph_for_record(&world.layout, ordinal)
                .unwrap();
            let image = raster
                .batch()
                .records()
                .iter()
                .find(|image| image.key() == record.key())
                .unwrap();
            let x = (positioned.origin_x_millipoints() as f64 * 1.25 / 1_000.0).floor()
                + f64::from(image.bearing().x_over_64()) / 64.0;
            let y = (positioned.origin_y_millipoints() as f64 * 1.25 / 1_000.0).floor()
                - f64::from(image.bearing().y_over_64()) / 64.0;
            Some([
                x.floor() as f32,
                y.floor() as f32,
                ((x + f64::from(image.extent().width())).ceil() - x.floor()) as f32,
                ((y + f64::from(image.extent().height())).ceil() - y.floor()) as f32,
            ])
        })
        .collect::<Vec<_>>();
    assert_eq!(expected_clears.len(), 3);
    let clears = operations
        .iter()
        .filter_map(|operation| match operation {
            Operation::Clear { bounds } => Some(*bounds),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        clears, expected_clears,
        "physical image damage is not scaled twice or widened across tabs"
    );
    assert_eq!(
        operations.len(),
        7,
        "three clear/text pairs and one occluder"
    );
    for (index, start) in [0, 2, 4].into_iter().enumerate() {
        assert!(matches!(operations[index * 2], Operation::Clear { .. }));
        let Operation::Glyph {
            run, vertex_color, ..
        } = operations[index * 2 + 1]
        else {
            panic!("each clear replays the ordinary text command first");
        };
        assert_eq!(run.mechanic(), expected[0]);
        assert_eq!(
            run.original_range(),
            UiTextOriginalRange::new(start, start + 1).unwrap()
        );
        assert_eq!(
            run.foreground().channels(),
            [255; 4],
            "immutable token attribution survives"
        );
        assert_eq!(vertex_color[..3], [0.0, 1.0, 0.0]);
        let alpha = (128.0 / 255.0) * (20_000.0 / 65_535.0);
        assert!(
            (vertex_color[3] - alpha).abs() < 0.000001,
            "composed opacity is applied once"
        );
    }
    assert!(matches!(
        operations[6],
        Operation::Solid {
            color: [30, 60, 90, 255],
            ..
        }
    ));
    world.with_native(world.attempt, |view| {
        let refreshed = atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [500, 60],
            )
            .unwrap();
        assert!(
            atlas
                .foreground_replay_commands(&commands, &order, view, [500, 60], &candidate)
                .is_err(),
            "old image observation must deny"
        );
        assert!(atlas
            .foreground_reconstruction_commands(&commands, &order, view, [500, 60], &candidate)
            .is_err());
        assert_eq!(
            atlas
                .foreground_reconstruction_commands(&commands, &order, view, [500, 60], &refreshed)
                .unwrap(),
            reconstructed
        );
        assert_eq!(
            atlas
                .foreground_replay_commands(&commands, &order, view, [500, 60], &refreshed)
                .unwrap(),
            operations
        );
    });
}

fn rectangle(
    text: &UiMountedSemanticTextMechanic,
    bounds: UiMountedCanonicalBox,
) -> UiMountedPaintCommand {
    let instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = UiMountedFilledRectMechanic::complete_from_runtime_mounting(
        UiMountedFilledRectCompletionInput {
            frame: text.frame(),
            surface: text.surface(),
            binding: text.binding(),
            mounted_instance: instance,
            node_receipt: UiMountedNodeReceiptIssuer::mint_for(text.frame())
                .unwrap()
                .receipt_for(instance),
            allocation_basis: text.allocation_basis(),
            bounds,
            clip_bounds: bounds,
            color: UiMountedRgba8::new(30, 60, 90, 255),
            layer_semantic_order: 2,
        },
    )
    .unwrap();
    UiMountedPaintCommand::FilledRect {
        identity: UiMountedPaintCommandIdentity::filled_rect(&mechanic),
        mechanic,
    }
}

fn bounds([x, y, width, height]: [f32; 4]) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
