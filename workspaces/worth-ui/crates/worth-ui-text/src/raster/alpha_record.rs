//! Alpha raster image admission and record construction.

use std::sync::Arc;

use sha2::Digest;

use swash::scale::image::{Content, Image};
use worth_ui_host_contract::UiCountedSha256;
use worth_ui_host_contract::{
    UiGlyphRasterContentDigest, UiGlyphRasterDemandRecord, UiGlyphRasterExtent,
};

use super::batch::{UiGlyphRasterAdmissionDenial, UiGlyphRasterRecord, UiGlyphRasterRecordInput};
use super::capacity::MAX_RASTER_EDGE;
use super::denial::UiGlyphRasterizationDenial;

#[derive(Clone, Copy)]
pub(super) struct RasterImageShape {
    pub(super) width: u32,
    pub(super) height: u32,
}

pub(super) fn validate_image(
    image: &Image,
) -> Result<RasterImageShape, UiGlyphRasterizationDenial> {
    if image.content != Content::Mask || image.data.is_empty() {
        return Err(UiGlyphRasterizationDenial::EmptyRaster);
    }
    let width = image.placement.width;
    let height = image.placement.height;
    if width == 0 || height == 0 || width > MAX_RASTER_EDGE || height > MAX_RASTER_EDGE {
        return Err(UiGlyphRasterizationDenial::ExtentExceeded);
    }
    let expected = usize::try_from(width)
        .ok()
        .and_then(|width| {
            usize::try_from(height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?;
    if image.data.len() != expected {
        return Err(UiGlyphRasterizationDenial::Record(
            UiGlyphRasterAdmissionDenial::ByteLengthMismatch {
                expected,
                actual: image.data.len(),
            },
        ));
    }
    Ok(RasterImageShape { width, height })
}

pub(super) fn content_digest(image: &Image) -> UiGlyphRasterContentDigest {
    UiGlyphRasterContentDigest::from_text_mechanics(UiCountedSha256::digest(&image.data).into())
}

pub(super) fn build_raster_record(
    record: UiGlyphRasterDemandRecord,
    image: Image,
    shape: RasterImageShape,
    digest: UiGlyphRasterContentDigest,
) -> Result<UiGlyphRasterRecord<super::source::UiAlphaRasterKind>, UiGlyphRasterizationDenial> {
    UiGlyphRasterRecord::<super::source::UiAlphaRasterKind>::from_text_mechanics(
        UiGlyphRasterRecordInput {
            key: record.key(),
            attribution: record.attribution(),
            bearing: worth_ui_host_contract::UiGlyphRasterBearing::from_sixty_fourths(
                image
                    .placement
                    .left
                    .checked_mul(64)
                    .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
                image
                    .placement
                    .top
                    .checked_mul(64)
                    .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
            ),
            extent: UiGlyphRasterExtent::new(shape.width, shape.height)
                .ok_or(UiGlyphRasterizationDenial::ExtentExceeded)?,
            stride: shape.width,
            pixels: Arc::from(image.data),
            digest,
        },
    )
    .map_err(UiGlyphRasterizationDenial::Record)
}
