//! Independent exact-output controls for the public alpha-raster boundary.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use swash::zeno::{Format, Vector};
use swash::{scale::*, FontRef, GlyphId};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontSlant, UiGlyphRasterLane, UiMountedCanonicalBox,
    UiMountedCanonicalBoxInput, UiMountedCoordinateSpace, UiMountedLogicalDamage, UiMountedRgba8,
    UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity, UiTextOriginalRange,
};

use super::{
    application_selection_tests::shape,
    application_test_world::{face, profile_collection_and_sources},
    UiApplicationFontPackDefinition,
};
use crate::{
    derive_glyph_raster_demand, rasterize_alpha_outline, UiFontFamilyStack,
    UiGlyphRasterDemandRequest, UiGlyphRasterPlacement, UiGlyphRasterScale, UiGlyphRasterSource,
    UiQualifiedTextLayout, UiTextFaceRequest,
};

#[derive(Debug, Eq, PartialEq)]
struct AlphaOracle {
    bearing: (i32, i32),
    extent: (u32, u32),
    digest: [u8; 32],
    pixels: Box<[u8]>,
}

#[test]
pub(super) fn public_alpha_raster_matches_exact_oracle_across_fractional_origins() {
    let layout = profile_layout("A");
    for (origin_x, origin_y, expected_x, expected_y) in [
        (0.0, 0.0, 0, 28),
        (0.25, 0.5, 16, 60),
        (0.75, 0.125, 48, 36),
        (-0.75, -20.0, -48, -36),
        (0.999, 0.0, 63, 28),
        (-0.999, -20.0, -63, -36),
    ] {
        assert_matches_pinned_swash_oracle(
            &layout,
            "A",
            (origin_x, origin_y),
            (expected_x, expected_y),
        );
    }
}

#[test]
pub(super) fn public_alpha_raster_matches_exact_variable_and_last_resort_oracle() {
    let (light, heavy) = variable_layouts();
    let last_resort = profile_layout("\u{0378}");
    assert_matches_pinned_swash_oracle(&light, "W", (0.0, 0.0), (0, 28));
    assert_matches_pinned_swash_oracle(&heavy, "W", (0.0, 0.0), (0, 28));
    assert_matches_pinned_swash_oracle(&last_resort, "\u{0378}", (0.0, 0.0), (0, 5));
}

fn assert_matches_pinned_swash_oracle(
    layout: &UiQualifiedTextLayout,
    source: &str,
    origin: (f32, f32),
    expected_origin_over_64: (i16, i16),
) {
    let damage = full_damage();
    let paint = paint(source);
    let demand = derive_glyph_raster_demand(
        layout,
        UiGlyphRasterDemandRequest {
            paint_spans: &[paint],
            selection: crate::UiGlyphRasterDemandSelection::LogicalDamage(&[damage]),
            scale: UiGlyphRasterScale::new(1_000, layout.view().text_scale_generation()).unwrap(),
            placement: UiGlyphRasterPlacement::from_mounted_logical(origin.0, origin.1).unwrap(),
            lane: UiGlyphRasterLane::Ordinary,
        },
    )
    .unwrap();
    let (record_index, demand_record) = demand
        .records()
        .iter()
        .copied()
        .enumerate()
        .find(|(_, record)| is_alpha(record.key().source()))
        .expect("qualified alpha demand");
    assert_eq!(
        (
            demand_record.key().fractional_origin().x_over_64(),
            demand_record.key().fractional_origin().y_over_64(),
        ),
        expected_origin_over_64,
        "mounted origin must become the exact independently expected 1/64-pixel key",
    );
    let raster = rasterize_alpha_outline(layout, &demand).unwrap();
    let production = raster
        .batch()
        .records()
        .iter()
        .find(|record| record.key() == demand_record.key())
        .expect("alpha oracle source produces one qualified raster");
    let observed = AlphaOracle {
        bearing: (
            production.bearing().x_over_64(),
            production.bearing().y_over_64(),
        ),
        extent: (production.extent().width(), production.extent().height()),
        digest: production.digest().bytes(),
        pixels: production.pixels().into(),
    };
    let reference = pinned_swash_reference(layout, demand_record.key());
    assert_eq!(observed, reference);
    let positioned = demand
        .positioned_glyph_for_record(layout, record_index)
        .unwrap();
    let origin_millipoints = [
        positioned.origin_x_millipoints() + demand.placement().origin_x_millipoints(),
        positioned.origin_y_millipoints() + demand.placement().origin_y_millipoints(),
    ];
    // Independent upstream image geometry at the integral pixel base. The
    // reference renderer above already used the fractional phase for its pixels.
    let expected = [
        (origin_millipoints[0] as f64 / 1_000.0).trunc() as f32 + reference.bearing.0 as f32 / 64.0,
        (origin_millipoints[1] as f64 / 1_000.0).trunc() as f32 - reference.bearing.1 as f32 / 64.0,
        reference.extent.0 as f32,
        reference.extent.1 as f32,
    ];
    assert_eq!(
        production.bearing().positioned_bounds(
            production.extent(),
            origin_millipoints,
            demand_record.key().dpi_milli(),
        ),
        Some(expected)
    );
}

/// The qualified profile pins Swash 0.2.10 as its upstream outline renderer.
/// This reference invokes that upstream API directly and never calls WORTH's
/// alpha raster wrapper, record builder, digest helper, or output bytes.
fn pinned_swash_reference(
    layout: &UiQualifiedTextLayout,
    key: worth_ui_host_contract::UiGlyphRasterKey,
) -> AlphaOracle {
    let resource = layout.artifact().face_resource(key.face()).unwrap();
    let face_index = usize::try_from(key.face().face_index()).unwrap();
    let face = FontRef::from_index(resource.bytes(), face_index).unwrap();
    let settings = key.variations().records().map(|variation| {
        (
            swash::Tag::from_be_bytes(variation.axis()),
            variation.value_milli() as f32 / 1_000.0,
        )
    });
    let mut context = ScaleContext::new();
    let mut scaler = context
        .builder(face)
        .size(pixels_per_em(key))
        .hint(true)
        .variations(settings)
        .build();
    let mut renderer = Render::new(&[Source::Outline]);
    renderer.format(Format::Alpha);
    renderer.offset(Vector::new(
        f32::from(key.fractional_origin().x_over_64()) / 64.0,
        f32::from(key.fractional_origin().y_over_64()) / 64.0,
    ));
    let image = renderer
        .render(
            &mut scaler,
            GlyphId::from(u16::try_from(key.glyph_id()).unwrap()),
        )
        .unwrap();
    AlphaOracle {
        bearing: (image.placement.left * 64, image.placement.top * 64),
        extent: (image.placement.width, image.placement.height),
        digest: Sha256::digest(&image.data).into(),
        pixels: image.data.into_boxed_slice(),
    }
}

fn pixels_per_em(key: worth_ui_host_contract::UiGlyphRasterKey) -> f32 {
    (key.size().millipoints() as f64 * f64::from(key.dpi_milli()) / 1_000_000.0) as f32
}

fn is_alpha(source: UiGlyphRasterSource) -> bool {
    matches!(
        source,
        UiGlyphRasterSource::AlphaOutline | UiGlyphRasterSource::LastResort
    )
}

fn variable_layouts() -> (UiQualifiedTextLayout, UiQualifiedTextLayout) {
    let (profile, sources) = profile_collection_and_sources();
    let definition = UiApplicationFontPackDefinition {
        name: Arc::from("alpha oracle variable family"),
        faces: Box::new([face(
            "Alpha Oracle Variable",
            Arc::clone(&sources["noto-sans-roman"]),
            0,
            UiFontSlant::Upright,
        )]),
    };
    let (collection, receipt, _) = profile
        .register_application_pack(UiFontCollectionGeneration::new(2).unwrap(), definition)
        .unwrap();
    let family = receipt.family("Alpha Oracle Variable").unwrap();
    let stack = || UiFontFamilyStack::new(Box::new([family])).unwrap();
    let collection = Arc::new(collection);
    let layout = |weight| {
        UiQualifiedTextLayout::layout(shape(
            &collection,
            stack(),
            UiTextFaceRequest::new(weight, 100_000, UiFontSlant::Upright).unwrap(),
            Box::new([]),
            "W",
        ))
        .unwrap()
    };
    (layout(100), layout(900))
}

fn profile_layout(source: &str) -> UiQualifiedTextLayout {
    crate::layout::tests::layout(source, 160_000, 1)
}

fn paint(source: &str) -> UiMountedTextForegroundSpan {
    UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::new(0, u32::try_from(source.len()).unwrap()).unwrap(),
        UiMountedRgba8::new(255, 255, 255, 255),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([0xA1; 32]),
    )
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
