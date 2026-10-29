//! Concrete glyph-raster identity admission.
//!
//! Record format is selected before these digests run, so alpha and color
//! batches share one compiled identity core.

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterAttribution, UiGlyphRasterBatchIdentity, UiGlyphRasterBearing,
    UiGlyphRasterContentDigest, UiGlyphRasterDemandIdentity, UiGlyphRasterExtent, UiGlyphRasterKey,
    UiGlyphRasterLane, UiQualifiedTextLayoutIdentity,
};

use super::demand::UiGlyphRasterScale;

pub(super) struct RasterIdentityFact {
    pub(super) key: UiGlyphRasterKey,
    pub(super) attribution: UiGlyphRasterAttribution,
    pub(super) bearing: UiGlyphRasterBearing,
    pub(super) extent: UiGlyphRasterExtent,
    pub(super) stride: u32,
    pub(super) digest: UiGlyphRasterContentDigest,
}

#[inline(never)]
pub(super) fn miss_identity(records: &[RasterIdentityFact]) -> UiGlyphRasterDemandIdentity {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-glyph-raster-miss-v1\0");
    digest.update(
        u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for record in records {
        update_key(&mut digest, record.key);
        update_attribution(&mut digest, record.attribution);
    }
    UiGlyphRasterDemandIdentity::from_text_mechanics(digest.finalize().into())
}

#[inline(never)]
pub(super) fn batch_identity(
    demand: UiGlyphRasterDemandIdentity,
    miss: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    lane: UiGlyphRasterLane,
    records: &[RasterIdentityFact],
) -> UiGlyphRasterBatchIdentity {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-glyph-raster-batch-v1\0");
    digest.update(demand.digest());
    digest.update(miss.digest());
    digest.update(layout.digest());
    digest.update(scale.dpi_milli().to_le_bytes());
    digest.update(scale.text_scale_generation().get().to_le_bytes());
    digest.update([lane_byte(lane)]);
    digest.update(
        u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for record in records {
        update_key(&mut digest, record.key);
        update_attribution(&mut digest, record.attribution);
        digest.update(record.bearing.x_over_64().to_le_bytes());
        digest.update(record.bearing.y_over_64().to_le_bytes());
        digest.update(record.extent.width().to_le_bytes());
        digest.update(record.extent.height().to_le_bytes());
        digest.update(record.stride.to_le_bytes());
        digest.update(record.digest.bytes());
    }
    UiGlyphRasterBatchIdentity::from_text_mechanics(digest.finalize().into())
}

fn update_key(digest: &mut UiCountedSha256, key: UiGlyphRasterKey) {
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
}

fn update_attribution(digest: &mut UiCountedSha256, attribution: UiGlyphRasterAttribution) {
    digest.update(attribution.layout().digest());
    digest.update(attribution.original_range().start().to_le_bytes());
    digest.update(attribution.original_range().end().to_le_bytes());
}

fn source_byte(source: worth_ui_host_contract::UiGlyphRasterSource) -> u8 {
    match source {
        worth_ui_host_contract::UiGlyphRasterSource::ColorOutline => 0,
        worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap => 1,
        worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline => 2,
        worth_ui_host_contract::UiGlyphRasterSource::LastResort => 3,
    }
}

fn lane_byte(lane: UiGlyphRasterLane) -> u8 {
    match lane {
        UiGlyphRasterLane::Ordinary => 0,
        UiGlyphRasterLane::Reconstruction => 1,
    }
}
