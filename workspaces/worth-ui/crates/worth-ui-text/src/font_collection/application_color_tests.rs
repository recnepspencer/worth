use std::sync::Arc;

use read_fonts::TableProvider;
use worth_ui_host_contract::{
    UiFontSlant, UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
    UiMountedLogicalDamage, UiMountedRgba8, UiMountedTextForegroundSpan,
    UiMountedTextPaintSpanIdentity, UiTextOriginalRange, UiTextScaleGeneration,
};

use super::{
    application_color_fixtures::{
        colr_v0, colr_v0_for_glyph, colr_v1, colr_v1_composite, colr_v1_for_glyph,
        colr_v1_gradient, corrupt_cbdt_png, corrupt_cblc_location, cpal, maxp_glyph_count,
        rename_tables, sbix, sbix_for_glyphs, with_tables,
    },
    application_selection_tests::selected_faces,
    application_test_world::{assert_pack_denial, face, layout, profile_collection_and_sources},
    UiApplicationFontPackDefinition, UiFontCollectionAdmissionDenial,
};
use crate::{
    derive_glyph_raster_demand, rasterize_intrinsic_color, UiFontFamilyStack,
    UiGlyphRasterDemandBatch, UiGlyphRasterDemandRequest, UiGlyphRasterLane,
    UiGlyphRasterPlacement, UiGlyphRasterScale, UiGlyphRasterSource, UiQualifiedTextLayout,
    UiTextFaceRequest,
};

#[test]
pub(super) fn owned_color_emoji_bytes_are_selected_as_one_complete_application_cluster() {
    let (profile, sources) = profile_collection_and_sources();
    let (collection, receipt, _) = profile
        .register_application_pack(
            worth_ui_host_contract::UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from("owned application color emoji"),
                faces: Box::new([face(
                    "Application Emoji",
                    Arc::clone(&sources["noto-color-emoji"]),
                    0,
                    UiFontSlant::Upright,
                )]),
            },
        )
        .unwrap();
    let family = receipt.family("Application Emoji").unwrap();
    let face = receipt.faces()[0].identity();
    assert!(receipt.faces()[0].has_intrinsic_color());

    let selected = selected_faces(
        &Arc::new(collection),
        UiFontFamilyStack::new(Box::new([family])).unwrap(),
        UiTextFaceRequest::regular(),
        "\u{1F469}\u{200D}\u{1F4BB}",
    );
    assert_eq!(&*selected, &[face]);
}

#[test]
pub(super) fn every_qualified_color_table_format_crosses_public_pack_admission() {
    let (mut collection, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let glyph_count = maxp_glyph_count(outline);
    let sources = [
        (
            "COLRv0",
            with_tables(outline, &[(b"COLR", colr_v0()), (b"CPAL", cpal())]),
        ),
        (
            "COLRv1",
            with_tables(outline, &[(b"COLR", colr_v1()), (b"CPAL", cpal())]),
        ),
        ("CBDT/CBLC", Arc::clone(&sources["noto-color-emoji"])),
        (
            "sbix",
            with_tables(outline, &[(b"sbix", sbix(glyph_count))]),
        ),
    ];
    for (index, (format, bytes)) in sources.into_iter().enumerate() {
        assert!(
            harfrust::FontRef::from_index(&bytes, 0).is_ok(),
            "{format} parser"
        );
        let generation = worth_ui_host_contract::UiFontCollectionGeneration::new(
            u64::try_from(index).unwrap() + 2,
        )
        .unwrap();
        let (successor, receipt, _) = collection
            .register_application_pack(
                generation,
                UiApplicationFontPackDefinition {
                    name: Arc::from(format),
                    faces: Box::new([face(
                        &format!("Application {format}"),
                        bytes,
                        0,
                        UiFontSlant::Upright,
                    )]),
                },
            )
            .unwrap_or_else(|denial| panic!("{format} public admission failed: {denial:?}"));
        assert!(receipt.faces()[0].has_intrinsic_color(), "{format}");
        collection = successor;
    }
}

#[test]
pub(super) fn qualified_color_source_twins_keep_outline_precedence_and_bitmap_identity() {
    let (_, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let bytes = with_tables(
        outline,
        &[
            (b"COLR", colr_v0()),
            (b"CPAL", cpal()),
            (b"sbix", sbix(maxp_glyph_count(outline))),
        ],
    );
    let font = harfrust::FontRef::from_index(&bytes, 0).unwrap();
    let coverage = super::color_glyph::validate(&font).unwrap();
    let source_for = |glyph| {
        coverage
            .iter()
            .find(|candidate| candidate.glyph_id() == glyph)
            .map(|candidate| candidate.source())
    };
    assert_eq!(
        source_for(1),
        Some(crate::layout_artifact::UiQualifiedTextColorSource::Outline)
    );
    assert_eq!(
        source_for(2),
        Some(crate::layout_artifact::UiQualifiedTextColorSource::Bitmap)
    );
}

#[test]
pub(super) fn qualified_sbix_png_and_one_hop_dupe_rasterize_as_intrinsic_color() {
    let (profile, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let glyph_id = read_fonts::FontRef::from_index(outline, 0)
        .unwrap()
        .cmap()
        .unwrap()
        .map_codepoint('A')
        .unwrap()
        .to_u32();
    let glyph_count = maxp_glyph_count(outline);
    let direct_glyph = if glyph_id == 1 { 2 } else { 1 };
    let bytes = with_tables(
        outline,
        &[(
            b"sbix",
            sbix_for_glyphs(
                glyph_count,
                direct_glyph,
                Some(u16::try_from(glyph_id).unwrap()),
            ),
        )],
    );
    let (collection, receipt, _) = profile
        .register_application_pack(
            worth_ui_host_contract::UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from("sbix raster owner"),
                faces: Box::new([face("sbix raster face", bytes, 0, UiFontSlant::Upright)]),
            },
        )
        .unwrap();
    let layout = layout(
        Arc::new(collection),
        receipt.family("sbix raster face").unwrap(),
        "A",
    );
    let demand = color_demand(&layout);
    assert!(demand
        .records()
        .iter()
        .any(|record| record.key().source() == UiGlyphRasterSource::ColorBitmap));
    let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
    assert!(raster.batch().records().iter().any(|record| {
        record.key().source() == UiGlyphRasterSource::ColorBitmap
            && record.pixels().chunks_exact(4).any(|pixel| pixel[3] != 0)
    }));
}

#[test]
pub(super) fn qualified_colrv1_composite_and_gradient_enums_cross_public_admission() {
    let (mut collection, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    for (index, table) in [colr_v1_composite(3), colr_v1_gradient(2)]
        .into_iter()
        .enumerate()
    {
        let generation =
            worth_ui_host_contract::UiFontCollectionGeneration::new(index as u64 + 2).unwrap();
        let (successor, receipt, _) = collection
            .register_application_pack(
                generation,
                UiApplicationFontPackDefinition {
                    name: Arc::from(format!("qualified COLRv1 enums {index}")),
                    faces: Box::new([face(
                        &format!("Qualified COLRv1 {index}"),
                        with_tables(outline, &[(b"COLR", table), (b"CPAL", cpal())]),
                        0,
                        UiFontSlant::Upright,
                    )]),
                },
            )
            .unwrap_or_else(|denial| panic!("qualified COLRv1 enum {index}: {denial:?}"));
        assert!(receipt.faces()[0].has_intrinsic_color());
        collection = successor;
    }
}

#[test]
pub(super) fn qualified_colr_owner_produces_intrinsic_color_pixels() {
    for name in ["COLRv0", "COLRv1"] {
        let (_, sources) = profile_collection_and_sources();
        let outline = &sources["noto-sans-roman"];
        let glyph_id = read_fonts::FontRef::from_index(outline, 0)
            .unwrap()
            .cmap()
            .unwrap()
            .map_codepoint('A')
            .unwrap()
            .to_u32();
        let table = if name == "COLRv0" {
            colr_v0_for_glyph(u16::try_from(glyph_id).unwrap())
        } else {
            colr_v1_for_glyph(u16::try_from(glyph_id).unwrap())
        };
        assert_colr_owner_produces_pixels(name, outline, glyph_id, table);
    }
}

fn assert_colr_owner_produces_pixels(
    name: &str,
    outline: &Arc<[u8]>,
    glyph_id: u32,
    table: Vec<u8>,
) {
    if name == "COLRv1" {
        let bytes = with_tables(outline, &[(b"COLR", table.clone()), (b"CPAL", cpal())]);
        let font = harfrust::FontRef::from_index(&bytes, 0).unwrap();
        let coverage = super::color_glyph::validate(&font).unwrap();
        assert!(
            coverage
                .iter()
                .any(|candidate| u32::from(candidate.glyph_id()) == glyph_id),
            "gid {glyph_id}: {:?}",
            coverage.iter().collect::<Vec<_>>()
        );
    }
    let (profile, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    let (collection, receipt, _) = profile
        .register_application_pack(
            worth_ui_host_contract::UiFontCollectionGeneration::new(2).unwrap(),
            UiApplicationFontPackDefinition {
                name: Arc::from(format!("{name} raster owner")),
                faces: Box::new([face(
                    &format!("{name} raster face"),
                    with_tables(outline, &[(b"COLR", table), (b"CPAL", cpal())]),
                    0,
                    UiFontSlant::Upright,
                )]),
            },
        )
        .unwrap_or_else(|denial| panic!("{name}: {denial:?}"));
    let layout = layout(
        Arc::new(collection),
        receipt.family(&format!("{name} raster face")).unwrap(),
        "A",
    );
    let demand = color_demand(&layout);
    assert!(
        demand
            .records()
            .iter()
            .any(|record| record.key().source() == UiGlyphRasterSource::ColorOutline),
        "{name}: {:?}",
        demand
            .records()
            .iter()
            .map(|record| (record.key().glyph_id(), record.key().source()))
            .collect::<Vec<_>>()
    );
    let raster = rasterize_intrinsic_color(&layout, &demand).unwrap();
    assert!(!raster.batch().records().is_empty(), "{name}");
    assert!(raster.batch().records().iter().all(|record| {
        record.key().source() == UiGlyphRasterSource::ColorOutline
            && record.pixels().chunks_exact(4).any(|pixel| pixel[3] != 0)
    }));
}

#[test]
pub(super) fn unknown_colrv1_composite_and_gradient_enums_deny_before_publication() {
    let (profile, sources) = profile_collection_and_sources();
    let outline = &sources["noto-sans-roman"];
    for (name, table) in [
        ("unknown COLRv1 composite mode", colr_v1_composite(0xFF)),
        ("unknown COLRv1 gradient extend", colr_v1_gradient(0xFF)),
    ] {
        let bytes = with_tables(outline, &[(b"COLR", table), (b"CPAL", cpal())]);
        assert!(harfrust::FontRef::from_index(&bytes, 0).is_ok(), "{name}");
        assert_pack_denial(
            &profile,
            UiApplicationFontPackDefinition {
                name: Arc::from(name),
                faces: Box::new([face(name, bytes, 0, UiFontSlant::Upright)]),
            },
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        );
    }
}

#[test]
pub(super) fn repository_color_emoji_requires_resolvable_locations_and_intact_png_chunks() {
    let (profile, sources) = profile_collection_and_sources();
    let emoji = Arc::clone(&sources["noto-color-emoji"]);
    for (name, bytes) in [
        ("corrupt CBLC location", corrupt_cblc_location(&emoji)),
        ("corrupt CBDT image", corrupt_cbdt_png(&emoji)),
    ] {
        assert!(harfrust::FontRef::from_index(&bytes, 0).is_ok());
        assert_pack_denial(
            &profile,
            UiApplicationFontPackDefinition {
                name: Arc::from(name),
                faces: Box::new([face("Application Emoji", bytes, 0, UiFontSlant::Upright)]),
            },
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        );
    }
}

#[test]
pub(super) fn malformed_colr_and_sbix_tables_cannot_hide_behind_a_parseable_outline_font() {
    let (profile, sources) = profile_collection_and_sources();
    let outline = Arc::clone(&sources["noto-sans-roman"]);
    let malformed_colr = rename_tables(&outline, &[(*b"name", *b"COLR"), (*b"post", *b"CPAL")]);
    let malformed_sbix = rename_tables(&outline, &[(*b"name", *b"sbix")]);
    for (name, bytes) in [
        ("malformed COLR graph", malformed_colr),
        ("malformed sbix strikes", malformed_sbix),
    ] {
        assert!(harfrust::FontRef::from_index(&bytes, 0).is_ok());
        assert_pack_denial(
            &profile,
            UiApplicationFontPackDefinition {
                name: Arc::from(name),
                faces: Box::new([face("Application Color", bytes, 0, UiFontSlant::Upright)]),
            },
            UiFontCollectionAdmissionDenial::MalformedColorFontTables,
        );
    }
}

pub(super) fn color_demand(layout: &UiQualifiedTextLayout) -> UiGlyphRasterDemandBatch {
    color_demand_at_dpi(layout, 1_000)
}

pub(super) fn color_demand_at_dpi(
    layout: &UiQualifiedTextLayout,
    dpi_milli: u32,
) -> UiGlyphRasterDemandBatch {
    derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[UiMountedTextForegroundSpan::from_runtime_mounting(
                UiTextOriginalRange::new(0, 1).unwrap(),
                UiMountedRgba8::new(255, 255, 255, 255),
                UiMountedTextPaintSpanIdentity::from_runtime_mounting([9; 32]),
            )],
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(&[full_damage()]),
            scale: UiGlyphRasterScale::new(dpi_milli, UiTextScaleGeneration::new(1).unwrap())
                .unwrap(),
            placement: UiGlyphRasterPlacement::default(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap()
}

fn full_damage() -> UiMountedLogicalDamage {
    UiMountedLogicalDamage::from_runtime_mounting(
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: -4.0,
            y: -24.0,
            width: 220.0,
            height: 80.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap(),
    )
}
