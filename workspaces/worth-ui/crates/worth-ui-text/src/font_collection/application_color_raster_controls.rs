//! Production-boundary controls for application color-font raster semantics.

use std::sync::Arc;

use read_fonts::TableProvider;
use worth_ui_host_contract::{UiFontCollectionGeneration, UiFontSlant};

use super::{
    application_color_fixtures::{
        colr_v0_layers_for_glyph, cpal_colors, maxp_glyph_count, sbix_strikes_for_glyph,
        unsupported_cbdt_bit_depth, with_tables,
    },
    application_color_tests::color_demand_at_dpi,
    application_test_world::{assert_pack_denial, face, layout, profile_collection_and_sources},
    UiApplicationFontPackDefinition, UiFontCollectionAdmissionDenial,
};
use crate::{rasterize_intrinsic_color, UiGlyphRasterSource};

#[test]
pub(super) fn colrv0_layers_use_linear_premultiplied_order() {
    let (outline, glyph) = outline_and_a_glyph();
    let colors = cpal_colors(&[[255, 0, 0, 128], [0, 0, 255, 128]]);
    let red_then_blue = raster_pixels(
        &outline,
        colr_v0_layers_for_glyph(glyph, &[0, 1]),
        colors.clone(),
    );
    let blue_then_red = raster_pixels(&outline, colr_v0_layers_for_glyph(glyph, &[1, 0]), colors);
    let first = fullest_pixel(&red_then_blue);
    let second = fullest_pixel(&blue_then_red);
    assert_ne!(
        red_then_blue, blue_then_red,
        "layer order is raster authority"
    );
    assert!((190..=193).contains(&first[3]));
    assert!((190..=193).contains(&second[3]));
    assert_eq!([first[0], first[2]], [second[2], second[0]]);
    assert!(first[0].max(first[2]) >= 126 && first[0].min(first[2]) >= 62);
}

#[test]
pub(super) fn bitmap_raster_selects_the_globally_nearest_qualified_strike() {
    let (outline, glyph) = outline_and_a_glyph();
    let bytes = with_tables(
        &outline,
        &[(
            b"sbix",
            sbix_strikes_for_glyph(
                maxp_glyph_count(&outline),
                glyph,
                &[(64, [0, 0, 255, 255]), (16, [255, 0, 0, 255])],
            ),
        )],
    );
    let near_small =
        raster_application_face_at_dpi(Arc::clone(&bytes), UiGlyphRasterSource::ColorBitmap, 1_000);
    let near_large = raster_application_face_at_dpi(bytes, UiGlyphRasterSource::ColorBitmap, 4_000);
    assert_eq!(fullest_pixel(&near_small), [255, 0, 0, 255]);
    assert_eq!(fullest_pixel(&near_large), [0, 0, 255, 255]);
}

#[test]
pub(super) fn unsupported_cbdt_pixel_formats_deny_before_application_pack_publication() {
    let (profile, sources) = profile_collection_and_sources();
    let bytes = unsupported_cbdt_bit_depth(&sources["noto-color-emoji"]);
    assert_pack_denial(
        &profile,
        UiApplicationFontPackDefinition {
            name: Arc::from("unsupported CBDT bit depth"),
            faces: Box::new([face(
                "unsupported CBDT face",
                bytes,
                0,
                UiFontSlant::Upright,
            )]),
        },
        UiFontCollectionAdmissionDenial::UnsupportedColorFontTable,
    );
}

fn outline_and_a_glyph() -> (Arc<[u8]>, u16) {
    let (_, sources) = profile_collection_and_sources();
    let outline = Arc::clone(&sources["noto-sans-roman"]);
    let glyph = read_fonts::FontRef::from_index(&outline, 0)
        .unwrap()
        .cmap()
        .unwrap()
        .map_codepoint('A')
        .unwrap()
        .to_u32();
    (outline, u16::try_from(glyph).unwrap())
}

fn raster_pixels(outline: &[u8], colr: Vec<u8>, cpal: Vec<u8>) -> Vec<u8> {
    raster_application_face(
        with_tables(outline, &[(b"COLR", colr), (b"CPAL", cpal)]),
        UiGlyphRasterSource::ColorOutline,
    )
}

fn raster_application_face(bytes: Arc<[u8]>, source: UiGlyphRasterSource) -> Vec<u8> {
    raster_application_face_at_dpi(bytes, source, 1_000)
}

fn raster_application_face_at_dpi(
    bytes: Arc<[u8]>,
    source: UiGlyphRasterSource,
    dpi_milli: u32,
) -> Vec<u8> {
    let (profile, _) = profile_collection_and_sources();
    let (collection, receipt, _) = profile
        .register_application_pack(
            UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from("color raster control"),
                faces: Box::new([face(
                    "color raster control face",
                    bytes,
                    0,
                    UiFontSlant::Upright,
                )]),
            },
        )
        .unwrap();
    let layout = layout(
        Arc::new(collection),
        receipt.family("color raster control face").unwrap(),
        "A",
    );
    let demand = color_demand_at_dpi(&layout, dpi_milli);
    let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
    raster
        .batch()
        .records()
        .iter()
        .find(|record| record.key().source() == source)
        .expect("qualified color source produced a raster record")
        .pixels()
        .to_vec()
}

fn fullest_pixel(pixels: &[u8]) -> [u8; 4] {
    pixels
        .as_chunks::<4>()
        .0
        .iter()
        .max_by_key(|pixel| pixel[3])
        .copied()
        .unwrap()
}
