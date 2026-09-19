//! Content-framed identity for one exact glyph-raster demand batch.

use sha2::{Digest, Sha256};
use worth_ui_host_contract::{
    UiGlyphRasterDemandIdentity, UiGlyphRasterDemandRecord, UiGlyphRasterDemandScope,
    UiGlyphRasterLane, UiGlyphRasterSource, UiQualifiedTextLayoutIdentity,
};

use super::demand::UiGlyphRasterScale;
use super::placement::UiGlyphRasterPlacement;

const DEMAND_IDENTITY_DOMAIN: &[u8] = b"worth-ui-glyph-raster-demand-v2\0";

pub(super) fn demand_identity(
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    placement: UiGlyphRasterPlacement,
    lane: UiGlyphRasterLane,
    scope: UiGlyphRasterDemandScope,
    records: &[UiGlyphRasterDemandRecord],
) -> UiGlyphRasterIdentity {
    let mut digest = Sha256::new();
    digest.update(DEMAND_IDENTITY_DOMAIN);
    digest.update(layout.digest());
    digest.update(scale.dpi_milli().to_le_bytes());
    digest.update(scale.text_scale_generation().get().to_le_bytes());
    digest.update(placement.origin_x_millipoints().to_le_bytes());
    digest.update(placement.origin_y_millipoints().to_le_bytes());
    digest.update([lane_byte(lane)]);
    digest.update([match scope {
        UiGlyphRasterDemandScope::CompleteLayout => 0,
        UiGlyphRasterDemandScope::DamageFiltered => 1,
    }]);
    digest.update(
        u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for record in records {
        let key = record.key();
        digest.update(key.font_collection_generation().get().to_le_bytes());
        digest.update(key.font_collection_lineage().digest());
        digest.update(key.profile_generation().get().to_le_bytes());
        digest.update(key.face().font_bytes_digest());
        digest.update(key.face().face_index().to_le_bytes());
        digest.update(key.face().selection_digest());
        digest.update(key.glyph_id().to_le_bytes());
        for variation in key.variations().records() {
            digest.update(variation.axis());
            digest.update(variation.value_milli().to_le_bytes());
        }
        digest.update([u8::try_from(key.variations().len()).unwrap_or(u8::MAX)]);
        digest.update(key.palette().index().to_le_bytes());
        digest.update(key.size().millipoints().to_le_bytes());
        digest.update([source_byte(key.source())]);
        digest.update(key.dpi_milli().to_le_bytes());
        digest.update(key.fractional_origin().x_over_64().to_le_bytes());
        digest.update(key.fractional_origin().y_over_64().to_le_bytes());
        digest.update(record.attribution().layout().digest());
        digest.update(record.attribution().original_range().start().to_le_bytes());
        digest.update(record.attribution().original_range().end().to_le_bytes());
        digest.update(record.extent().width().to_le_bytes());
        digest.update(record.extent().height().to_le_bytes());
        digest.update(record.staged_bytes().to_le_bytes());
    }
    UiGlyphRasterIdentity::from_text_mechanics(digest.finalize().into())
}

type UiGlyphRasterIdentity = UiGlyphRasterDemandIdentity;

fn lane_byte(lane: UiGlyphRasterLane) -> u8 {
    match lane {
        UiGlyphRasterLane::Ordinary => 0,
        UiGlyphRasterLane::Reconstruction => 1,
    }
}

fn source_byte(source: UiGlyphRasterSource) -> u8 {
    match source {
        UiGlyphRasterSource::ColorOutline => 0,
        UiGlyphRasterSource::ColorBitmap => 1,
        UiGlyphRasterSource::AlphaOutline => 2,
        UiGlyphRasterSource::LastResort => 3,
    }
}
