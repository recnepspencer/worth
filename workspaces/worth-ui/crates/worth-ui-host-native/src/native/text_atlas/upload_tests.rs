use super::*;
use crate::native::text_atlas::{
    capacity::source_channels,
    qualified_test_device,
    raster_upload::upload_shape_is_valid,
    upload_staging::{align_to_copy_row, source_matches_kind},
};
use crate::native::{UiNativeResourceCensus, UiNativeResourceRegistry};
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterSize,
    UiGlyphRasterSource, UiGlyphVariationCoordinates, UiQualifiedFontFaceIdentity,
    UiTextProfileGeneration,
};

fn key(source: UiGlyphRasterSource) -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([5; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([6; 32], 0),
        glyph_id: 5,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}

fn upload(source: UiGlyphRasterSource) -> UiNativeTextAtlasUpload {
    let channels = source_channels(source) as u32;
    UiNativeTextAtlasUpload::from_text_mechanics(
        key(source),
        2,
        2,
        2 * channels,
        vec![0; (4 * channels) as usize],
        [0; 32],
    )
}

#[test]
fn source_owners_and_copy_rows_are_separate_and_bounded() {
    assert!(source_matches_kind(
        UiGlyphRasterSource::AlphaOutline,
        UiNativeGpuAtlasKind::Alpha
    ));
    assert!(!source_matches_kind(
        UiGlyphRasterSource::ColorOutline,
        UiNativeGpuAtlasKind::Alpha
    ));
    assert_eq!(align_to_copy_row(1).unwrap(), 256);
    assert_eq!(align_to_copy_row(256).unwrap(), 256);
    assert!(upload_shape_is_valid(&upload(
        UiGlyphRasterSource::AlphaOutline
    )));
}

#[test]
pub(in crate::native::text_atlas) fn real_wgpu_upload_owns_pages_and_releases_staging() {
    let (device, queue, info) = qualified_test_device();
    assert_eq!(info.backend, crate::native::graphics::qualified_backend());
    let mut resources = UiNativeResourceRegistry::new();
    let mut pages = UiNativeTextAtlasGpuPages::new();
    pages
        .ensure_page(&device, &mut resources, UiNativeGpuAtlasKind::Alpha)
        .unwrap();
    assert_eq!(pages.page_count(UiNativeGpuAtlasKind::Alpha), 1);
    let alpha_upload = upload(UiGlyphRasterSource::AlphaOutline);
    let receipt = pages
        .upload(UiNativeTextAtlasGpuUploadRequest {
            device: &device,
            queue: &queue,
            resources: &mut resources,
            kind: UiNativeGpuAtlasKind::Alpha,
            page: 0,
            origin: [0, 0],
            upload: &alpha_upload,
        })
        .unwrap();
    assert_eq!(receipt.logical_bytes, 4);
    assert_eq!(receipt.physical_bytes, 512);
    assert_eq!(resources.current().atlas_staging_buffers, 1);
    pages.settle_pending(&mut resources);
    assert_eq!(resources.current().atlas_staging_buffers, 0);
    assert_eq!(resources.current().alpha_atlas_pages, 1);
    pages
        .ensure_page(&device, &mut resources, UiNativeGpuAtlasKind::Color)
        .unwrap();
    let color_upload = upload(UiGlyphRasterSource::ColorOutline);
    pages
        .upload(UiNativeTextAtlasGpuUploadRequest {
            device: &device,
            queue: &queue,
            resources: &mut resources,
            kind: UiNativeGpuAtlasKind::Color,
            page: 0,
            origin: [0, 0],
            upload: &color_upload,
        })
        .unwrap();
    pages.settle_pending(&mut resources);
    assert_eq!(resources.current().color_atlas_pages, 1);
    assert_eq!(resources.current().atlas_staging_buffers, 0);
    pages
        .try_close(&mut resources)
        .unwrap_or_else(|_| panic!("settled test uploads must close"));
    assert_eq!(resources.current(), UiNativeResourceCensus::default());
}
