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
        let input = admit_record(input, Kind::CHANNELS, Kind::source_matches)?;
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
    fn identity_fact(&self) -> super::batch_admission::RasterIdentityFact {
        super::batch_admission::RasterIdentityFact {
            key: self.key,
            attribution: self.attribution,
            bearing: self.bearing,
            extent: self.extent,
            stride: self.stride,
            digest: self.digest,
        }
    }

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
        let facts = identity_facts(&records);
        let miss = super::batch_admission::miss_identity(&facts);
        let batch =
            super::batch_admission::batch_identity(demand, miss, layout, scale, lane, &facts);
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
        let mut visit = Some(visit);
        let mut output = None;
        self.project_alpha_view(&mut |view| {
            output = Some(visit.take().expect("alpha batch view is visited once")(
                view,
            ));
        });
        output.expect("alpha batch view is visited once")
    }

    #[inline(never)]
    fn project_alpha_view(&self, visit: &mut dyn FnMut(UiAlphaRasterBatchView<'_, '_>)) {
        let views: Vec<_> = self.records.iter().map(Self::record_view).collect();
        visit(UiAlphaRasterBatchView::from_text_mechanics(
            self.demand,
            self.miss,
            self.batch,
            self.layout,
            self.lane,
            &views,
        ));
    }

    fn record_view(record: &UiGlyphRasterRecord<UiAlphaRasterKind>) -> UiAlphaRasterRecordView<'_> {
        record.as_view()
    }
}

impl UiGlyphRasterBatch<UiColorRasterKind> {
    pub fn with_view<R>(&self, visit: impl FnOnce(UiColorRasterBatchView<'_, '_>) -> R) -> R {
        let mut visit = Some(visit);
        let mut output = None;
        self.project_color_view(&mut |view| {
            output = Some(visit.take().expect("color batch view is visited once")(
                view,
            ));
        });
        output.expect("color batch view is visited once")
    }

    #[inline(never)]
    fn project_color_view(&self, visit: &mut dyn FnMut(UiColorRasterBatchView<'_, '_>)) {
        let views: Vec<_> = self.records.iter().map(Self::record_view).collect();
        visit(UiColorRasterBatchView::from_text_mechanics(
            self.demand,
            self.miss,
            self.batch,
            self.layout,
            self.lane,
            &views,
        ));
    }

    fn record_view(record: &UiGlyphRasterRecord<UiColorRasterKind>) -> UiColorRasterRecordView<'_> {
        record.as_view()
    }
}

fn identity_facts<Kind>(
    records: &[UiGlyphRasterRecord<Kind>],
) -> Vec<super::batch_admission::RasterIdentityFact> {
    records
        .iter()
        .map(UiGlyphRasterRecord::identity_fact)
        .collect()
}

#[inline(never)]
fn admit_record(
    input: UiGlyphRasterRecordInput,
    channels: usize,
    source_matches: fn(worth_ui_host_contract::UiGlyphRasterSource) -> bool,
) -> Result<UiGlyphRasterRecordInput, UiGlyphRasterAdmissionDenial> {
    if !source_matches(input.key.source()) {
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
    match channels {
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
    Ok(input)
}
