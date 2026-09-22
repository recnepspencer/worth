//! Font-backed controls for admitted and denied intrinsic-color graphs.

use std::sync::Arc;

use read_fonts::TableProvider;
use worth_ui_host_contract::{UiFontCollectionGeneration, UiFontSlant};

use super::{
    application_color_fixtures::{
        colr_v0_layers_for_glyph, colr_v1_composite_palettes, colr_v1_gradient_palettes, cpal,
        cpal_colors, maxp_glyph_count, with_tables,
    },
    application_color_graph_fixtures::{
        cbdt_composite_font, colr_v1_bounded_then_unbounded_solid, colr_v1_clipped_solid,
        colr_v1_two_bounded_solids, colr_v1_unbounded_gradient, unsupported_sbix_font,
        CbdtCompositeTarget,
    },
    application_color_tests::color_demand,
    application_test_world::{assert_pack_denial, face, layout, profile_collection_and_sources},
    UiApplicationFontPackDefinition, UiFontCollectionAdmissionDenial,
};
use crate::{rasterize_intrinsic_color, UiGlyphRasterSource};

#[test]
pub(super) fn colrv1_gradient_and_composite_cross_the_font_backed_raster_boundary() {
    let (outline, glyph) = outline_and_a_glyph();
    let palettes = cpal_colors(&[[255, 0, 0, 255], [0, 0, 255, 255]]);
    let gradient = raster_application_face(with_tables(
        &outline,
        &[
            (b"COLR", colr_v1_gradient_palettes(glyph, 0, 1)),
            (b"CPAL", palettes.clone()),
        ],
    ));
    assert!(gradient
        .as_chunks::<4>()
        .0
        .iter()
        .any(|pixel| { pixel[3] != 0 && pixel[0] > pixel[2] }));
    assert!(gradient
        .as_chunks::<4>()
        .0
        .iter()
        .any(|pixel| { pixel[3] != 0 && pixel[2] > pixel[0] }));

    let composite = raster_application_face(with_tables(
        &outline,
        &[
            (b"COLR", colr_v1_composite_palettes(glyph, 1, 0, 1)),
            (b"CPAL", palettes),
        ],
    ));
    let fullest = fullest_pixel(&composite);
    assert!((187..=189).contains(&fullest[0]));
    assert_eq!(fullest[1], 0);
    assert_eq!(fullest[2], 0);
    assert!((126..=129).contains(&fullest[3]));
}

#[test]
pub(super) fn colrv1_nonseparable_modes_match_independent_w3c_vectors_after_srgb_storage() {
    let (outline, glyph) = outline_and_a_glyph();
    let palettes = cpal_colors(&[[230, 90, 140, 255], [60, 210, 110, 255]]);
    for (mode, expected) in [
        (24, [172, 134, 121, 191]),
        (25, [126, 161, 104, 191]),
        (26, [175, 133, 121, 191]),
        (27, [126, 154, 101, 191]),
    ] {
        let pixels = raster_application_face(with_tables(
            &outline,
            &[
                (b"COLR", colr_v1_composite_palettes(glyph, mode, 0, 1)),
                (b"CPAL", palettes.clone()),
            ],
        ));
        let actual = fullest_pixel(&pixels);
        for (actual, expected) in actual.into_iter().zip(expected) {
            assert!(
                actual.abs_diff(expected) <= 1,
                "mode {mode}: {actual} != {expected}"
            );
        }
    }
}

#[test]
pub(super) fn cbdt_composite_crosses_admission_layout_demand_and_raster() {
    let (outline, glyph) = outline_and_a_glyph();
    let pixels = raster_application_face(cbdt_composite_font(
        &outline,
        glyph,
        CbdtCompositeTarget::Child,
    ));
    assert_eq!(pixels, [120, 0, 0, 48]);
}

#[test]
pub(super) fn cbdt_missing_target_cycle_and_sbix_jpg_tiff_deny_atomically() {
    let (profile, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let glyph = read_fonts::FontRef::from_index(outline, 0)
        .unwrap()
        .cmap()
        .unwrap()
        .map_codepoint('A')
        .unwrap()
        .to_u32() as u16;
    for (name, bytes, denial) in [
        (
            "CBDT missing composite target",
            cbdt_composite_font(outline, glyph, CbdtCompositeTarget::Missing),
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        ),
        (
            "CBDT composite cycle",
            cbdt_composite_font(outline, glyph, CbdtCompositeTarget::Cycle),
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        ),
        (
            "sbix jpg",
            unsupported_sbix_font(outline, glyph, *b"jpg "),
            UiFontCollectionAdmissionDenial::UnsupportedColorFontTable,
        ),
        (
            "sbix tiff",
            unsupported_sbix_font(outline, glyph, *b"tiff"),
            UiFontCollectionAdmissionDenial::UnsupportedColorFontTable,
        ),
    ] {
        assert_pack_denial(
            &profile,
            UiApplicationFontPackDefinition {
                name: Arc::from(name),
                faces: Box::new([face(name, bytes, 0, UiFontSlant::Upright)]),
            },
            denial,
        );
    }
}

#[test]
pub(super) fn current_color_and_every_unbounded_colrv1_root_deny_atomically() {
    let (profile, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let glyph = read_fonts::FontRef::from_index(outline, 0)
        .unwrap()
        .cmap()
        .unwrap()
        .map_codepoint('A')
        .unwrap()
        .to_u32() as u16;
    let next = glyph + 1;
    assert!(next < maxp_glyph_count(outline));
    let (lawful, receipt, _) = profile
        .register_application_pack(
            UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from("COLRv1 bounded multi-root control"),
                faces: Box::new([face(
                    "COLRv1 bounded multi-root control",
                    with_tables(
                        outline,
                        &[
                            (b"COLR", colr_v1_two_bounded_solids(glyph, next)),
                            (b"CPAL", cpal()),
                        ],
                    ),
                    0,
                    UiFontSlant::Upright,
                )]),
            },
        )
        .expect("the two-root table is lawful when every root is bounded");
    assert!(receipt
        .family("COLRv1 bounded multi-root control")
        .is_some());
    assert_eq!(lawful.application_packs().len(), 1);
    let (profile, _) = profile_collection_and_sources();
    for (name, colr) in [
        (
            "COLRv0 current color",
            colr_v0_layers_for_glyph(glyph, &[0xFFFF]),
        ),
        ("COLRv1 current color", colr_v1_clipped_solid(glyph, 0xFFFF)),
        (
            "COLRv1 unbounded gradient",
            colr_v1_unbounded_gradient(glyph),
        ),
        (
            "COLRv1 later unbounded solid root",
            colr_v1_bounded_then_unbounded_solid(glyph, next),
        ),
    ] {
        assert_pack_denial(
            &profile,
            UiApplicationFontPackDefinition {
                name: Arc::from(name),
                faces: Box::new([face(
                    name,
                    with_tables(outline, &[(b"COLR", colr), (b"CPAL", cpal())]),
                    0,
                    UiFontSlant::Upright,
                )]),
            },
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        );
    }
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

fn raster_application_face(bytes: Arc<[u8]>) -> Vec<u8> {
    let (profile, _) = profile_collection_and_sources();
    let (collection, receipt, _) = profile
        .register_application_pack(
            UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from("color graph control"),
                faces: Box::new([face(
                    "color graph control face",
                    bytes,
                    0,
                    UiFontSlant::Upright,
                )]),
            },
        )
        .unwrap();
    let layout = layout(
        Arc::new(collection),
        receipt.family("color graph control face").unwrap(),
        "A",
    );
    let demand = color_demand(&layout);
    let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
    raster
        .batch()
        .records()
        .iter()
        .find(|record| {
            matches!(
                record.key().source(),
                UiGlyphRasterSource::ColorOutline | UiGlyphRasterSource::ColorBitmap
            )
        })
        .expect("qualified color graph produced one raster")
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
