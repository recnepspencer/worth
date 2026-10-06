use std::sync::Arc;

use super::*;
use crate::raster::batch::UiGlyphRasterRecordInput;
use crate::raster::cost::UiGlyphRasterLaneCostInput;
use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterDemandIdentity,
    UiGlyphRasterFractionalOrigin, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize,
    UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity, UiQualifiedTextLayoutIdentity,
    UiTextOriginalRange, UiTextProfileGeneration, UiTextScaleGeneration,
};

fn sample_key(source: UiGlyphRasterSource) -> UiGlyphRasterKey {
    admit_raster_key(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([3; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([2; 32], 0),
        glyph_id: 7,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(14_000).unwrap(),
        source,
        dpi_milli: 1_500,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}

fn sample_record_input(source: UiGlyphRasterSource) -> UiGlyphRasterRecordInput {
    let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([1; 32]);
    let attribution = UiGlyphRasterAttribution::from_text_mechanics(
        layout,
        UiTextOriginalRange::new(0, 4).unwrap(),
    );
    UiGlyphRasterRecordInput {
        key: sample_key(source),
        attribution,
        bearing: UiGlyphRasterBearing::from_sixty_fourths(0, 0),
        extent: UiGlyphRasterExtent::new(2, 2).unwrap(),
        stride: 2,
        pixels: Arc::from([255; 4]),
        digest: UiGlyphRasterContentDigest::from_text_mechanics(Sha256::digest([255; 4]).into()),
    }
}

#[test]
fn alpha_records_preserve_digest_and_batch_membership() {
    let layout = sample_layout();
    let alpha = UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(sample_record_input(
        UiGlyphRasterSource::AlphaOutline,
    ));
    let batch = UiGlyphRasterBatch::from_text_mechanics(
        UiGlyphRasterDemandIdentity::from_text_mechanics([0; 32]),
        layout,
        UiGlyphRasterScale::new(1_500, UiTextScaleGeneration::new(1).unwrap()).unwrap(),
        UiGlyphRasterLane::Ordinary,
        [alpha.unwrap()],
    )
    .unwrap();
    assert_eq!(batch.records().len(), 1);
    let expected_digest: [u8; 32] = Sha256::digest([255; 4]).into();
    assert_eq!(batch.records()[0].digest().bytes(), expected_digest);
    batch.with_view(|view| assert_eq!(view.records().len(), 1));
}

#[test]
fn alpha_batch_view_carries_exact_demand_miss_and_batch_identity() {
    let layout = sample_layout();
    let scale = UiGlyphRasterScale::new(1_500, UiTextScaleGeneration::new(1).unwrap()).unwrap();
    let first = UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(sample_record_input(
        UiGlyphRasterSource::AlphaOutline,
    ))
    .unwrap();
    let second = UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(
        sample_record_input(UiGlyphRasterSource::AlphaOutline),
    )
    .unwrap();
    let left = UiGlyphRasterBatch::from_text_mechanics(
        UiGlyphRasterDemandIdentity::from_text_mechanics([1; 32]),
        layout,
        scale,
        UiGlyphRasterLane::Ordinary,
        [first],
    )
    .unwrap();
    let right = UiGlyphRasterBatch::from_text_mechanics(
        UiGlyphRasterDemandIdentity::from_text_mechanics([2; 32]),
        layout,
        scale,
        UiGlyphRasterLane::Ordinary,
        [second],
    )
    .unwrap();
    assert_ne!(left.demand_identity(), right.demand_identity());
    assert_eq!(left.miss_identity(), right.miss_identity());
    assert_ne!(left.batch_identity(), right.batch_identity());
    left.with_view(|view| {
        assert_eq!(view.demand_identity(), left.demand_identity());
        assert_eq!(view.miss_identity(), left.miss_identity());
        assert_eq!(view.batch_identity(), left.batch_identity());
    });
}

#[test]
fn color_records_reject_alpha_byte_shape() {
    let color = UiGlyphRasterRecord::<UiColorRasterKind>::from_text_mechanics(sample_record_input(
        UiGlyphRasterSource::ColorOutline,
    ));
    assert_eq!(
        color.err(),
        Some(UiGlyphRasterAdmissionDenial::ByteLengthMismatch {
            expected: 16,
            actual: 4,
        })
    );
}

#[test]
fn raster_cost_keeps_observed_work_in_its_lane() {
    let cost = UiGlyphRasterLaneCost::from_text_mechanics(UiGlyphRasterLaneCostInput {
        layout_visits: 4,
        outer_traversals: 4,
        validation_checks: 3,
        provenance_checks: 2,
        demanded_glyphs: 3,
        face_resource_lookups: 3,
        outline_evaluations: 2,
        bitmap_source_evaluations: 0,
        retained_scans: 0,
        cache_hits: 1,
        cache_misses: 2,
        rasterized_glyphs: 2,
        rasterized_texels: 40,
        produced_bytes: 40,
    });
    assert_eq!(cost.rasterized_glyphs(), 2);
    assert_eq!(cost.cache_misses(), 2);
}

#[test]
fn raster_records_reject_wrong_content_digest() {
    let mut wrong_digest = sample_record_input(UiGlyphRasterSource::AlphaOutline);
    wrong_digest.digest = UiGlyphRasterContentDigest::from_text_mechanics([0; 32]);
    assert_eq!(
        UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(wrong_digest).err(),
        Some(UiGlyphRasterAdmissionDenial::ContentDigestMismatch)
    );
}

fn sample_layout() -> UiQualifiedTextLayoutIdentity {
    UiQualifiedTextLayoutIdentity::from_text_mechanics([1; 32])
}
