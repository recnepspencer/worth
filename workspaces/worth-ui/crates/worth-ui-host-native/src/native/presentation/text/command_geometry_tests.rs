//! Run-to-command placement controls using explicit, inert atlas metadata.

use worth_ui_host_contract::*;

use super::{command_for_run, UiNativeGpuAtlasKind, UiNativeTextAtlasEntryView};

#[test]
fn glyph_commands_position_raster_images_once_and_crop_matching_texels() {
    for (source, kind) in [
        (
            UiGlyphRasterSource::AlphaOutline,
            UiNativeGpuAtlasKind::Alpha,
        ),
        (
            UiGlyphRasterSource::ColorOutline,
            UiNativeGpuAtlasKind::Color,
        ),
        (
            UiGlyphRasterSource::ColorBitmap,
            UiNativeGpuAtlasKind::Color,
        ),
    ] {
        for (origin, phase, expected_target, expected_uv) in [
            (
                [2_750, 5_250],
                [48, 16],
                [1.0, 2.0, 8.0, 12.0],
                [0.25, 0.5, 0.25, 0.375],
            ),
            (
                [-750, -250],
                [-48, -16],
                [0.0, 0.0, 7.0, 9.0],
                [0.28125, 0.59375, 0.21875, 0.28125],
            ),
            (
                [999, -999],
                [63, -63],
                [0.0, 0.0, 7.0, 9.0],
                [0.28125, 0.59375, 0.21875, 0.28125],
            ),
        ] {
            let run = run(source, origin, phase);
            let entry = UiNativeTextAtlasEntryView {
                kind,
                page: 2,
                origin: [8, 16],
                extent: [8, 12],
                page_extent: [32, 32],
                bearing: UiGlyphRasterBearing::from_sixty_fourths(-64, 192),
            };
            let command = command_for_run(run, entry, [32, 32]).unwrap().unwrap();
            assert_eq!(command.target, expected_target);
            assert_eq!(command.texture_uv, expected_uv);
            assert_eq!(command.run, run);
            assert_eq!(command.atlas_kind, kind);
            assert_eq!(command.atlas_page, 2);
        }
    }
}

fn run(source: UiGlyphRasterSource, origin: [i64; 2], phase: [i16; 2]) -> UiGlyphRunView {
    let key = UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([1; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 0),
        glyph_id: 11,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(phase[0], phase[1]),
    })
    .unwrap();
    let mechanic = UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
        UiMountedInstanceIdentity::mint_unbound().unwrap(),
        0,
        None,
    );
    UiGlyphRunView::from_text_mechanics(UiGlyphRunViewInput {
        mechanic,
        layout: UiQualifiedTextLayoutIdentity::from_text_mechanics([3; 32]),
        paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([4; 32]),
        original_range: UiTextOriginalRange::new(0, 1).unwrap(),
        foreground: UiMountedRgba8::new(30, 80, 120, 200),
        raster_key: key,
        origin_x_millipoints: origin[0],
        origin_y_millipoints: origin[1],
        line_index: 0,
        visual_run_index: 0,
        clip_bounds: UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: -4.0,
            y: -4.0,
            width: 36.0,
            height: 36.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap(),
        layer_semantic_order: 7,
    })
}
