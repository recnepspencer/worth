//! Exact borrowed raster-batch admission for one native atlas miss group.

use std::collections::{HashMap, HashSet};

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiAlphaRasterBatchView, UiColorRasterBatchView, UiGlyphRasterBatchIdentity,
    UiGlyphRasterBatchSink, UiGlyphRasterBatchSubmissionDenial, UiGlyphRasterDemandIdentity,
    UiGlyphRasterKey, UiGlyphRasterLane, UiQualifiedTextLayoutIdentity,
};

#[cfg(test)]
use worth_ui_host_contract::{UiAlphaRasterRecordView, UiColorRasterRecordView};

use crate::native::text_atlas::UiNativeTextAtlasUpload;

use super::text_atlas_rasterization::MissGroup;

pub(super) struct UploadSink {
    expected: HashMap<UiGlyphRasterKey, worth_ui_host_contract::UiGlyphRasterDemandRecord>,
    expected_demand: UiGlyphRasterDemandIdentity,
    expected_layout: UiQualifiedTextLayoutIdentity,
    expected_lane: UiGlyphRasterLane,
    expected_dpi_milli: u32,
    expected_text_scale: u64,
    seen: HashSet<UiGlyphRasterKey>,
    pub(super) uploads: Vec<UiNativeTextAtlasUpload>,
}

impl UploadSink {
    pub(super) fn new(group: &MissGroup) -> Self {
        Self {
            expected: group
                .records
                .iter()
                .copied()
                .map(|record| (record.key(), record))
                .collect(),
            expected_demand: group.demand,
            expected_layout: group.layout,
            expected_lane: group.lane,
            expected_dpi_milli: group.dpi_milli,
            expected_text_scale: group.text_scale.get(),
            seen: HashSet::new(),
            uploads: Vec::new(),
        }
    }

    pub(super) fn finish(
        &self,
    ) -> Result<(), worth_ui_host_contract::UiGlyphRasterTransactionDenial> {
        if self.seen == self.expected.keys().copied().collect() {
            Ok(())
        } else {
            Err(worth_ui_host_contract::UiGlyphRasterTransactionDenial::RasterBatchMismatch)
        }
    }

    fn validate_batch(
        &self,
        demand: UiGlyphRasterDemandIdentity,
        layout: UiQualifiedTextLayoutIdentity,
        lane: UiGlyphRasterLane,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        if demand != self.expected_demand {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongDemand);
        }
        if layout != self.expected_layout {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongLayout);
        }
        if lane != self.expected_lane {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongLayout);
        }
        Ok(())
    }

    fn validate_batch_identity(
        &self,
        identity: UiGlyphRasterBatchIdentity,
        miss: UiGlyphRasterDemandIdentity,
        records: impl Iterator<Item = RasterRecordInput>,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        let records = records.collect::<Vec<_>>();
        if miss != miss_identity(&records) {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongMiss);
        }
        let expected = batch_identity(
            self.expected_demand,
            miss,
            self.expected_layout,
            self.expected_dpi_milli,
            self.expected_text_scale,
            self.expected_lane,
            records.into_iter(),
        );
        (identity == expected)
            .then_some(())
            .ok_or(UiGlyphRasterBatchSubmissionDenial::WrongBatch)
    }

    fn admit_record(
        &mut self,
        record: RasterRecordInput,
        pixels: &[u8],
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        let expected = self
            .expected
            .get(&record.key)
            .ok_or(UiGlyphRasterBatchSubmissionDenial::WrongMiss)?;
        let reserved = expected.extent();
        if expected.attribution() != record.attribution
            || record.extent.width() > reserved.width()
            || record.extent.height() > reserved.height()
        {
            return Err(UiGlyphRasterBatchSubmissionDenial::Malformed);
        }
        let actual_digest: [u8; 32] = UiCountedSha256::digest(pixels).into();
        if actual_digest != record.digest {
            return Err(UiGlyphRasterBatchSubmissionDenial::Malformed);
        }
        worth_ui_host_contract::record_presentation_map_inserts(1);
        if !self.seen.insert(record.key) {
            return Err(UiGlyphRasterBatchSubmissionDenial::Duplicate);
        }
        self.uploads
            .push(UiNativeTextAtlasUpload::with_bearing_from_text_mechanics(
                record.key,
                record.bearing,
                record.extent.width(),
                record.extent.height(),
                record.stride,
                pixels.to_vec().into_boxed_slice(),
                record.digest,
            ));
        Ok(())
    }
}

impl UiGlyphRasterBatchSink for UploadSink {
    fn submit_alpha(
        &mut self,
        batch: UiAlphaRasterBatchView<'_, '_>,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        self.validate_batch(
            batch.demand_identity(),
            batch.layout_identity(),
            batch.lane(),
        )?;
        self.validate_batch_identity(
            batch.batch_identity(),
            batch.miss_identity(),
            batch.records().iter().map(RasterRecordInput::from_alpha),
        )?;
        if batch.records().iter().any(|record| {
            !matches!(
                record.key().source(),
                worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline
                    | worth_ui_host_contract::UiGlyphRasterSource::LastResort
            )
        }) {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongSource);
        }
        for record in batch.records() {
            self.admit_record(RasterRecordInput::from_alpha(record), record.pixels())?;
        }
        Ok(())
    }

    fn submit_color(
        &mut self,
        batch: UiColorRasterBatchView<'_, '_>,
    ) -> Result<(), UiGlyphRasterBatchSubmissionDenial> {
        self.validate_batch(
            batch.demand_identity(),
            batch.layout_identity(),
            batch.lane(),
        )?;
        self.validate_batch_identity(
            batch.batch_identity(),
            batch.miss_identity(),
            batch.records().iter().map(RasterRecordInput::from_color),
        )?;
        if batch.records().iter().any(|record| {
            !matches!(
                record.key().source(),
                worth_ui_host_contract::UiGlyphRasterSource::ColorOutline
                    | worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap
            )
        }) {
            return Err(UiGlyphRasterBatchSubmissionDenial::WrongSource);
        }
        for record in batch.records() {
            self.admit_record(RasterRecordInput::from_color(record), record.pixels())?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
struct RasterRecordInput {
    key: UiGlyphRasterKey,
    attribution: worth_ui_host_contract::UiGlyphRasterAttribution,
    bearing: worth_ui_host_contract::UiGlyphRasterBearing,
    extent: worth_ui_host_contract::UiGlyphRasterExtent,
    stride: u32,
    digest: [u8; 32],
}

impl RasterRecordInput {
    fn from_alpha(record: &worth_ui_host_contract::UiAlphaRasterRecordView<'_>) -> Self {
        Self {
            key: record.key(),
            attribution: record.attribution(),
            bearing: record.bearing(),
            extent: record.extent(),
            stride: record.stride(),
            digest: record.digest().bytes(),
        }
    }

    fn from_color(record: &worth_ui_host_contract::UiColorRasterRecordView<'_>) -> Self {
        Self {
            key: record.key(),
            attribution: record.attribution(),
            bearing: record.bearing(),
            extent: record.extent(),
            stride: record.stride(),
            digest: record.digest().bytes(),
        }
    }
}

fn miss_identity(records: &[RasterRecordInput]) -> UiGlyphRasterDemandIdentity {
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-glyph-raster-miss-v1\0");
    digest.update(
        u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for record in records {
        digest.update(batch_key_bytes(record.key));
        digest.update(record.attribution.layout().digest());
        digest.update(record.attribution.original_range().start().to_le_bytes());
        digest.update(record.attribution.original_range().end().to_le_bytes());
    }
    UiGlyphRasterDemandIdentity::from_text_mechanics(digest.finalize().into())
}

fn batch_identity(
    demand: UiGlyphRasterDemandIdentity,
    miss: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    dpi_milli: u32,
    text_scale: u64,
    lane: UiGlyphRasterLane,
    records: impl Iterator<Item = RasterRecordInput>,
) -> UiGlyphRasterBatchIdentity {
    let records = records.collect::<Vec<_>>();
    let mut digest = UiCountedSha256::new();
    digest.update(b"worth-ui-glyph-raster-batch-v1\0");
    digest.update(demand.digest());
    digest.update(miss.digest());
    digest.update(layout.digest());
    digest.update(dpi_milli.to_le_bytes());
    digest.update(text_scale.to_le_bytes());
    digest.update([lane_byte(lane)]);
    digest.update(
        u64::try_from(records.len())
            .unwrap_or(u64::MAX)
            .to_le_bytes(),
    );
    for record in records {
        digest.update(batch_key_bytes(record.key));
        digest.update(record.attribution.layout().digest());
        digest.update(record.attribution.original_range().start().to_le_bytes());
        digest.update(record.attribution.original_range().end().to_le_bytes());
        digest.update(record.bearing.x_over_64().to_le_bytes());
        digest.update(record.bearing.y_over_64().to_le_bytes());
        digest.update(record.extent.width().to_le_bytes());
        digest.update(record.extent.height().to_le_bytes());
        digest.update(record.stride.to_le_bytes());
        digest.update(record.digest);
    }
    UiGlyphRasterBatchIdentity::from_text_mechanics(digest.finalize().into())
}

fn lane_byte(lane: UiGlyphRasterLane) -> u8 {
    match lane {
        UiGlyphRasterLane::Ordinary => 0,
        UiGlyphRasterLane::Reconstruction => 1,
    }
}

fn batch_key_bytes(key: UiGlyphRasterKey) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(&key.font_collection_generation().get().to_le_bytes());
    bytes.extend_from_slice(&key.font_collection_lineage().digest());
    bytes.extend_from_slice(&key.profile_generation().get().to_le_bytes());
    bytes.extend_from_slice(&key.face().font_bytes_digest());
    bytes.extend_from_slice(&key.face().face_index().to_le_bytes());
    bytes.extend_from_slice(&key.face().selection_digest());
    bytes.extend_from_slice(&key.glyph_id().to_le_bytes());
    for variation in key.variations().records() {
        bytes.extend_from_slice(&variation.axis());
        bytes.extend_from_slice(&variation.value_milli().to_le_bytes());
    }
    bytes.push(u8::try_from(key.variations().len()).unwrap_or(u8::MAX));
    bytes.extend_from_slice(&key.palette().index().to_le_bytes());
    bytes.extend_from_slice(&key.size().millipoints().to_le_bytes());
    bytes.push(match key.source() {
        worth_ui_host_contract::UiGlyphRasterSource::ColorOutline => 0,
        worth_ui_host_contract::UiGlyphRasterSource::ColorBitmap => 1,
        worth_ui_host_contract::UiGlyphRasterSource::AlphaOutline => 2,
        worth_ui_host_contract::UiGlyphRasterSource::LastResort => 3,
    });
    bytes.extend_from_slice(&key.dpi_milli().to_le_bytes());
    bytes.extend_from_slice(&key.fractional_origin().x_over_64().to_le_bytes());
    bytes.extend_from_slice(&key.fractional_origin().y_over_64().to_le_bytes());
    bytes
}

#[cfg(test)]
pub(super) fn expected_alpha_batch_identity(
    demand: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    dpi_milli: u32,
    text_scale: u64,
    lane: UiGlyphRasterLane,
    records: &[UiAlphaRasterRecordView<'_>],
) -> (UiGlyphRasterDemandIdentity, UiGlyphRasterBatchIdentity) {
    let inputs = records
        .iter()
        .map(RasterRecordInput::from_alpha)
        .collect::<Vec<_>>();
    let miss = miss_identity(&inputs);
    let batch = batch_identity(
        demand,
        miss,
        layout,
        dpi_milli,
        text_scale,
        lane,
        inputs.into_iter(),
    );
    (miss, batch)
}

#[cfg(test)]
pub(super) fn expected_color_batch_identity(
    demand: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    dpi_milli: u32,
    text_scale: u64,
    lane: UiGlyphRasterLane,
    records: &[UiColorRasterRecordView<'_>],
) -> (UiGlyphRasterDemandIdentity, UiGlyphRasterBatchIdentity) {
    let inputs = records
        .iter()
        .map(RasterRecordInput::from_color)
        .collect::<Vec<_>>();
    let miss = miss_identity(&inputs);
    let batch = batch_identity(
        demand,
        miss,
        layout,
        dpi_milli,
        text_scale,
        lane,
        inputs.into_iter(),
    );
    (miss, batch)
}
