//! Reconstruction rebuilds adopted paint without retinting token-owned ranges.
use super::foreground_coverage_test_world::CoverageWorld;
use worth_ui_host_contract::*;
use worth_ui_host_native::{
    UiNativeTextForegroundAtlasModel, UiNativeTextReplayOperation as Operation,
};

#[test]
fn reconstruction_keeps_adopted_and_token_ranges_distinct_at_fractional_dpi() {
    let world = CoverageWorld::with_physical_geometry(
        "WW",
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0.0,
        [0.0, 0.0, 400.0, 48.0],
        None,
        &[(UiSemanticTextSlot::Value, 0.0)],
        ([0, 255, 0, 128], 20_000),
        (160.0, 1_250),
    )
    .with_token_suffix(1);
    let text = world.fragment.text_candidates()[0].clone();
    let identity = UiMountedPaintCommandIdentity::semantic_text(&text);
    let commands = [UiMountedPaintCommand::SemanticText {
        identity,
        mechanic: text,
    }];
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut atlas = UiNativeTextForegroundAtlasModel::new();
    let (candidate, _) = world.with_native(world.attempt, |view| {
        atlas
            .rasterize_with_simulated_submission(
                &world.fragment,
                view,
                &world.foreground,
                [500, 60],
            )
            .unwrap()
    });
    let (operations, report) = world.with_native(world.attempt, |view| {
        atlas
            .foreground_reconstruction_commands(&commands, &order, view, [500, 60], &candidate)
            .unwrap()
    });
    assert_eq!(report.rasterized_glyphs(), 0);
    assert_eq!(operations.len(), 3);
    assert!(matches!(
        operations[0],
        Operation::Clear {
            bounds: [0.0, 0.0, 500.0, 60.0]
        }
    ));
    let Operation::Glyph {
        run: adopted,
        vertex_color: adopted_color,
        ..
    } = operations[1]
    else {
        panic!("adopted image");
    };
    let Operation::Glyph {
        run: token,
        vertex_color: token_color,
        ..
    } = operations[2]
    else {
        panic!("token image");
    };
    assert_eq!(
        adopted.original_range(),
        UiTextOriginalRange::new(0, 1).unwrap()
    );
    assert_eq!(
        token.original_range(),
        UiTextOriginalRange::new(1, 2).unwrap()
    );
    assert_eq!(adopted.foreground().channels(), [255; 4]);
    assert_eq!(token.foreground().channels(), [100, 150, 200, 255]);
    assert_eq!(adopted_color[..3], [0.0, 1.0, 0.0]);
    assert!((adopted_color[3] - (128.0 / 255.0) * (20_000.0 / 65_535.0)).abs() < 0.000001);
    // Vertex RGB is linear light; these are the independently evaluated sRGB
    // transfer values for token bytes 100, 150, and 200.
    let expected_token = [0.127_437_68, 0.304_987_31, 0.577_580_45, 1.0];
    for (actual, expected) in token_color.into_iter().zip(expected_token) {
        assert!((actual - expected).abs() < 0.000001);
    }
}
