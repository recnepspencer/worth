//! Phase 5 raster authority contracts owned by text mechanics.
//!
//! Text-owned qualified glyph demand, alpha-outline, and intrinsic-color raster.
//!
//! This module stops before atlas effects, GPU work, and presentation
//! settlement.

mod alpha;
mod alpha_admission;
mod alpha_record;
mod alpha_transaction_admission;
mod alpha_transaction_completion;
mod batch;
mod cache;
mod capacity;
mod color;
mod cost;
mod demand;
mod demand_candidate;
mod demand_geometry;
mod demand_identity;
mod denial;
mod key;
mod placement;
mod planning_geometry;
mod qualified_raster_admission;
mod source;

pub use alpha::{
    rasterize_alpha_outline, rasterize_alpha_outline_selection,
    rasterize_alpha_outline_selection_cached, rasterize_alpha_outline_transaction,
    UiAlphaRasterTransaction, UiAlphaRasterization,
};
pub use alpha_admission::{admit_alpha_outline, UiAlphaRasterAdmission};
pub use alpha_transaction_admission::{
    admit_alpha_outline_transaction, UiAlphaRasterTransactionAdmission,
};
pub use alpha_transaction_completion::{
    UiAlphaRasterBatchCompletion, UiAlphaRasterTransactionCompletion,
};
pub use batch::{
    UiAlphaRasterBatch, UiColorRasterBatch, UiGlyphRasterAdmissionDenial, UiGlyphRasterBatch,
    UiGlyphRasterRecord,
};
pub use cache::UiGlyphRasterCache;
pub use color::admission::{
    admit_intrinsic_color, admit_intrinsic_color_transaction, UiColorRasterAdmission,
    UiColorRasterTransactionAdmission,
};
pub use color::completion::{UiColorRasterBatchCompletion, UiColorRasterTransactionCompletion};
pub use color::{
    rasterize_intrinsic_color, rasterize_intrinsic_color_selection,
    rasterize_intrinsic_color_selection_cached, rasterize_intrinsic_color_transaction,
    UiColorRasterTransaction, UiColorRasterization,
};
pub use cost::{UiGlyphRasterCost, UiGlyphRasterLaneCost};
pub use demand::{
    derive_glyph_raster_demand, UiGlyphRasterDemandBatch, UiGlyphRasterDemandDenial,
    UiGlyphRasterDemandRequest, UiGlyphRasterDemandSelection, UiGlyphRasterScale,
};
pub use denial::UiGlyphRasterizationDenial;
pub use key::admit_raster_key;
pub use placement::UiGlyphRasterPlacement;
pub use source::{UiAlphaRasterKind, UiColorRasterKind, UiGlyphRasterFormat};
pub use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterBearing, UiGlyphRasterContentDigest,
    UiGlyphRasterExtent, UiGlyphRasterKey, UiGlyphRasterLane, UiGlyphRasterSource,
};

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::raster::batch::UiGlyphRasterRecordInput;
    use crate::raster::cost::UiGlyphRasterLaneCostInput;
    use sha2::{Digest, Sha256};
    use worth_ui_host_contract::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterDemandIdentity,
        UiGlyphRasterFractionalOrigin, UiGlyphRasterKeyInput, UiGlyphRasterPalette,
        UiGlyphRasterSize, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
        UiQualifiedTextLayoutIdentity, UiTextOriginalRange, UiTextProfileGeneration,
        UiTextScaleGeneration,
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
            digest: UiGlyphRasterContentDigest::from_text_mechanics(
                Sha256::digest([255; 4]).into(),
            ),
        }
    }

    #[test]
    fn alpha_records_preserve_digest_and_batch_membership() {
        let layout = sample_layout();
        let alpha = UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(
            sample_record_input(UiGlyphRasterSource::AlphaOutline),
        );
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
        let first = UiGlyphRasterRecord::<UiAlphaRasterKind>::from_text_mechanics(
            sample_record_input(UiGlyphRasterSource::AlphaOutline),
        )
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
        let color = UiGlyphRasterRecord::<UiColorRasterKind>::from_text_mechanics(
            sample_record_input(UiGlyphRasterSource::ColorOutline),
        );
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
}

#[cfg(test)]
mod demand_alpha_tests;
#[cfg(test)]
pub(crate) mod demand_identity_tests;
#[cfg(test)]
mod demand_ligature_tests;
#[cfg(test)]
mod demand_scope_tests;

#[cfg(test)]
mod alpha_transaction_tests;
