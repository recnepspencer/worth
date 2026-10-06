//! Text-owned alpha and intrinsic-color raster records and batches.
//!
//! Constructors stay crate-owned. Host consumers receive borrowed views.

use std::sync::Arc;

use sha2::Digest;

use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiAlphaRasterBatchView, UiAlphaRasterRecordView, UiColorRasterBatchView,
    UiColorRasterRecordView, UiGlyphRasterAttribution, UiGlyphRasterBatchIdentity,
    UiGlyphRasterBearing, UiGlyphRasterContentDigest, UiGlyphRasterDemandIdentity,
    UiGlyphRasterExtent, UiGlyphRasterKey, UiGlyphRasterLane, UiGlyphRasterRecordViewInput,
    UiGlyphRasterViewDenial, UiQualifiedTextLayoutIdentity,
};

use super::capacity::{MAX_BATCH_RECORDS, MAX_RASTER_EDGE, MAX_STAGED_BYTES};
use super::demand::UiGlyphRasterScale;
use super::source::{UiAlphaRasterKind, UiColorRasterKind, UiGlyphRasterFormat};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiGlyphRasterAdmissionDenial {
    ByteLengthOverflow,
    ByteLengthMismatch { expected: usize, actual: usize },
    StrideMismatch,
    SourceFormatMismatch,
    ForeignLayout,
    ForeignKey,
    ExtentExceeded,
    BatchCapacityExceeded,
    StagedByteCapacityExceeded,
    ContentDigestMismatch,
}

pub struct UiGlyphRasterRecord<Kind> {
    key: UiGlyphRasterKey,
    attribution: UiGlyphRasterAttribution,
    bearing: UiGlyphRasterBearing,
    extent: UiGlyphRasterExtent,
    stride: u32,
    pixels: Arc<[u8]>,
    digest: UiGlyphRasterContentDigest,
    _format: std::marker::PhantomData<Kind>,
}

pub(crate) struct UiGlyphRasterRecordInput {
    pub key: UiGlyphRasterKey,
    pub attribution: UiGlyphRasterAttribution,
    pub bearing: UiGlyphRasterBearing,
    pub extent: UiGlyphRasterExtent,
    pub stride: u32,
    pub pixels: Arc<[u8]>,
    pub digest: UiGlyphRasterContentDigest,
}

pub struct UiGlyphRasterBatch<Kind> {
    demand: UiGlyphRasterDemandIdentity,
    miss: UiGlyphRasterDemandIdentity,
    batch: UiGlyphRasterBatchIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    lane: UiGlyphRasterLane,
    records: Box<[UiGlyphRasterRecord<Kind>]>,
}

pub type UiAlphaRasterBatch = UiGlyphRasterBatch<UiAlphaRasterKind>;
pub type UiColorRasterBatch = UiGlyphRasterBatch<UiColorRasterKind>;

impl From<UiGlyphRasterViewDenial> for UiGlyphRasterAdmissionDenial {
    fn from(denial: UiGlyphRasterViewDenial) -> Self {
        match denial {
            UiGlyphRasterViewDenial::ByteLengthOverflow => Self::ByteLengthOverflow,
            UiGlyphRasterViewDenial::ByteLengthMismatch { expected, actual } => {
                Self::ByteLengthMismatch { expected, actual }
            }
            UiGlyphRasterViewDenial::StrideMismatch => Self::StrideMismatch,
        }
    }
}

impl<Kind: UiGlyphRasterFormat> UiGlyphRasterRecord<Kind> {
    pub(crate) fn from_text_mechanics(
        input: UiGlyphRasterRecordInput,
    ) -> Result<Self, UiGlyphRasterAdmissionDenial> {
        if !Kind::source_matches(input.key.source()) {
            return Err(UiGlyphRasterAdmissionDenial::SourceFormatMismatch);
        }
        if input.extent.width() > MAX_RASTER_EDGE || input.extent.height() > MAX_RASTER_EDGE {
            return Err(UiGlyphRasterAdmissionDenial::ExtentExceeded);
        }
        let view_input = UiGlyphRasterRecordViewInput {
            key: input.key,
            attribution: input.attribution,
            bearing: input.bearing,
            extent: input.extent,
            stride: input.stride,
            pixels: &input.pixels,
            digest: input.digest,
        };
        match Kind::CHANNELS {
            1 => {
                UiAlphaRasterRecordView::from_text_mechanics(view_input)?;
            }
            4 => {
                UiColorRasterRecordView::from_text_mechanics(view_input)?;
            }
            _ => return Err(UiGlyphRasterAdmissionDenial::ByteLengthOverflow),
        }
        let digest: [u8; 32] = UiCountedSha256::digest(&input.pixels).into();
        if digest != input.digest.bytes() {
            return Err(UiGlyphRasterAdmissionDenial::ContentDigestMismatch);
        }
        Ok(Self {
            key: input.key,
            attribution: input.attribution,
            bearing: input.bearing,
            extent: input.extent,
            stride: input.stride,
            pixels: input.pixels,
            digest: input.digest,
            _format: std::marker::PhantomData,
        })
    }

    pub const fn key(&self) -> UiGlyphRasterKey {
        self.key
    }
    pub const fn layout_identity(&self) -> UiQualifiedTextLayoutIdentity {
        self.attribution.layout()
    }
    pub const fn face_identity(&self) -> worth_ui_host_contract::UiQualifiedFontFaceIdentity {
        self.key.face()
    }
    pub const fn glyph_id(&self) -> u32 {
        self.key.glyph_id()
    }
    pub const fn cluster(&self) -> worth_ui_host_contract::UiTextOriginalRange {
        self.attribution.original_range()
    }
    pub const fn attribution(&self) -> UiGlyphRasterAttribution {
        self.attribution
    }
    pub const fn bearing(&self) -> UiGlyphRasterBearing {
        self.bearing
    }
    pub const fn extent(&self) -> UiGlyphRasterExtent {
        self.extent
    }
    pub const fn stride(&self) -> u32 {
        self.stride
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub(super) fn pixels_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.pixels)
    }
    pub const fn digest(&self) -> UiGlyphRasterContentDigest {
        self.digest
    }
}

impl UiGlyphRasterRecord<UiAlphaRasterKind> {
    pub fn as_view(&self) -> UiAlphaRasterRecordView<'_> {
        UiAlphaRasterRecordView::from_text_mechanics(self.view_input())
            .expect("admitted alpha record remains format-valid")
    }
}

impl UiGlyphRasterRecord<UiColorRasterKind> {
    pub fn as_view(&self) -> UiColorRasterRecordView<'_> {
        UiColorRasterRecordView::from_text_mechanics(self.view_input())
            .expect("admitted color record remains format-valid")
    }
}

impl<Kind> UiGlyphRasterRecord<Kind> {
    fn view_input(&self) -> UiGlyphRasterRecordViewInput<'_> {
        UiGlyphRasterRecordViewInput {
            key: self.key,
            attribution: self.attribution,
            bearing: self.bearing,
            extent: self.extent,
            stride: self.stride,
            pixels: &self.pixels,
            digest: self.digest,
        }
    }
}

impl<Kind: UiGlyphRasterFormat> UiGlyphRasterBatch<Kind> {
    pub(crate) fn from_text_mechanics(
        demand: UiGlyphRasterDemandIdentity,
        layout: UiQualifiedTextLayoutIdentity,
        scale: UiGlyphRasterScale,
        lane: UiGlyphRasterLane,
        records: impl IntoIterator<Item = UiGlyphRasterRecord<Kind>>,
    ) -> Result<Self, UiGlyphRasterAdmissionDenial> {
        let records: Box<[_]> = records.into_iter().collect();
        if records.len() > MAX_BATCH_RECORDS {
            return Err(UiGlyphRasterAdmissionDenial::BatchCapacityExceeded);
        }
        if records
            .iter()
            .any(|record| record.layout_identity() != layout)
        {
            return Err(UiGlyphRasterAdmissionDenial::ForeignLayout);
        }
        if records
            .iter()
            .any(|record| record.key.dpi_milli() != scale.dpi_milli())
        {
            return Err(UiGlyphRasterAdmissionDenial::ForeignKey);
        }
        let staged_bytes = records.iter().try_fold(0_u64, |total, record| {
            total.checked_add(u64::try_from(record.pixels.len()).ok()?)
        });
        if staged_bytes.is_none() {
            return Err(UiGlyphRasterAdmissionDenial::ByteLengthOverflow);
        }
        if staged_bytes.is_some_and(|bytes| bytes > MAX_STAGED_BYTES) {
            return Err(UiGlyphRasterAdmissionDenial::StagedByteCapacityExceeded);
        }
        let miss = miss_identity(&records);
        let batch = batch_identity(demand, miss, layout, scale, lane, &records);
        Ok(Self {
            demand,
            miss,
            batch,
            layout,
            scale,
            lane,
            records,
        })
    }

    pub const fn layout_identity(&self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub const fn demand_identity(&self) -> UiGlyphRasterDemandIdentity {
        self.demand
    }

    pub const fn miss_identity(&self) -> UiGlyphRasterDemandIdentity {
        self.miss
    }

    pub const fn batch_identity(&self) -> UiGlyphRasterBatchIdentity {
        self.batch
    }
    pub const fn scale(&self) -> UiGlyphRasterScale {
        self.scale
    }
    pub const fn lane(&self) -> UiGlyphRasterLane {
        self.lane
    }
    pub fn records(&self) -> &[UiGlyphRasterRecord<Kind>] {
        &self.records
    }
}

impl UiGlyphRasterBatch<UiAlphaRasterKind> {
    pub fn with_view<R>(&self, visit: impl FnOnce(UiAlphaRasterBatchView<'_, '_>) -> R) -> R {
        let views: Vec<_> = self.records.iter().map(Self::record_view).collect();
        visit(UiAlphaRasterBatchView::from_text_mechanics(
            self.demand,
            self.miss,
            self.batch,
            self.layout,
            self.lane,
            &views,
        ))
    }

    fn record_view(record: &UiGlyphRasterRecord<UiAlphaRasterKind>) -> UiAlphaRasterRecordView<'_> {
        record.as_view()
    }
}

impl UiGlyphRasterBatch<UiColorRasterKind> {
    pub fn with_view<R>(&self, visit: impl FnOnce(UiColorRasterBatchView<'_, '_>) -> R) -> R {
        let views: Vec<_> = self.records.iter().map(Self::record_view).collect();
        visit(UiColorRasterBatchView::from_text_mechanics(
            self.demand,
            self.miss,
            self.batch,
            self.layout,
            self.lane,
            &views,
        ))
    }

    fn record_view(record: &UiGlyphRasterRecord<UiColorRasterKind>) -> UiColorRasterRecordView<'_> {
        record.as_view()
    }
}

fn miss_identity<Kind>(records: &[UiGlyphRasterRecord<Kind>]) -> UiGlyphRasterDemandIdentity {
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

fn batch_identity<Kind>(
    demand: UiGlyphRasterDemandIdentity,
    miss: UiGlyphRasterDemandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    scale: UiGlyphRasterScale,
    lane: UiGlyphRasterLane,
    records: &[UiGlyphRasterRecord<Kind>],
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
