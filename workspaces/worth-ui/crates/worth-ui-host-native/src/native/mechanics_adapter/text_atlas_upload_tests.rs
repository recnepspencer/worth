use super::*;
use crate::native::text_atlas::{
    UiNativeTextAtlas, UiNativeTextAtlasCensus, UiNativeTextAtlasExternalOutcome,
    UiNativeTextAtlasPhysicalPosture, UiNativeTextAtlasPinTransition,
};
use crate::native::UiNativeResourceRegistry;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterAttribution,
    UiGlyphRasterDemandBatchView, UiGlyphRasterDemandBatchViewInput, UiGlyphRasterDemandIdentity,
    UiGlyphRasterDemandRecord, UiGlyphRasterExtent, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterLane, UiGlyphRasterPalette,
    UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    UiQualifiedFontFaceIdentity, UiTextOriginalRange, UiTextProfileGeneration,
    UiTextScaleGeneration,
};

#[test]
fn real_port_upload_is_coupled_to_production_plan_receipt_and_census() {
    let (device, queue, info) = crate::native::text_atlas::qualified_test_device();
    assert_eq!(info.backend, wgpu::Backend::Dx12);
    let alpha_key = key();
    let color_key = key_for_glyph_and_source(2, UiGlyphRasterSource::ColorOutline);
    let layout =
        worth_ui_host_contract::UiQualifiedTextLayoutIdentity::from_text_mechanics([6; 32]);
    let records = [
        UiGlyphRasterDemandRecord::from_text_mechanics(
            alpha_key,
            UiGlyphRasterAttribution::from_text_mechanics(
                layout,
                UiTextOriginalRange::new(0, 1).unwrap(),
            ),
            UiGlyphRasterExtent::new(2, 2).unwrap(),
        )
        .unwrap(),
        UiGlyphRasterDemandRecord::from_text_mechanics(
            color_key,
            UiGlyphRasterAttribution::from_text_mechanics(
                layout,
                UiTextOriginalRange::new(1, 2).unwrap(),
            ),
            UiGlyphRasterExtent::new(2, 2).unwrap(),
        )
        .unwrap(),
    ];
    let demand =
        UiGlyphRasterDemandBatchView::from_text_mechanics(UiGlyphRasterDemandBatchViewInput {
            identity: UiGlyphRasterDemandIdentity::from_text_mechanics([7; 32]),
            layout,
            dpi_milli: 1_000,
            text_scale: UiTextScaleGeneration::new(1).unwrap(),
            lane: UiGlyphRasterLane::Ordinary,
            scope: worth_ui_host_contract::UiGlyphRasterDemandScope::DamageFiltered,
            records: &records,
        })
        .unwrap();
    let atlas = UiNativeTextAtlas::new();
    let plan = atlas
        .plan_many(&[demand], &UiNativeTextAtlasPinTransition::default())
        .expect("production plan must admit the qualified upload");
    let uploads = [
        UiNativeTextAtlasUpload::from_text_mechanics(alpha_key, 2, 2, 2, vec![0; 4], [0; 32]),
        UiNativeTextAtlasUpload::from_text_mechanics(color_key, 2, 2, 8, vec![0; 16], [0; 32]),
    ];
    let mut resources = UiNativeResourceRegistry::new();
    let mut gpu = UiNativeTextAtlasGpuPages::new();
    RealTextAtlasUploadPort
        .submit_context(
            &mut UiNativeTextAtlasUploadContext {
                gpu: &mut gpu,
                device: &device,
                queue: &queue,
                resources: &mut resources,
            },
            UploadRequest {
                plan: &plan,
                uploads: &uploads,
            },
        )
        .expect("the production upload port must submit the qualified upload");
    assert_eq!(gpu.page_count(UiNativeGpuAtlasKind::Alpha), 1);
    assert_eq!(gpu.page_count(UiNativeGpuAtlasKind::Color), 1);
    assert_eq!(gpu.pending_count(), 1);
    assert_eq!(resources.current().atlas_staging_buffers, 1);
    gpu.settle_pending(&mut resources);
    assert_eq!(gpu.pending_count(), 0);
    assert_eq!(resources.current().atlas_staging_buffers, 0);
    let receipt = match atlas.settle(plan, &uploads, UiNativeTextAtlasExternalOutcome::Submitted) {
        crate::native::text_atlas::UiNativeTextAtlasCommitOutcome::Committed(receipt) => receipt,
        other => panic!("qualified production transaction must settle: {other:?}"),
    };
    assert_eq!(receipt.misses, 2);
    let census = UiNativeTextAtlasCensus::from_snapshot_with_posture(
        atlas.snapshot(),
        0,
        UiNativeTextAtlasPhysicalPosture {
            alpha_pages: gpu.page_count(UiNativeGpuAtlasKind::Alpha),
            color_pages: gpu.page_count(UiNativeGpuAtlasKind::Color),
            staging_buffers: resources.current().atlas_staging_buffers,
            upload_submissions: gpu.pending_count(),
            in_flight_transactions: 0,
        },
    );
    assert_eq!(census.alpha_pages, 1);
    assert_eq!(census.alpha_entries, 1);
    assert_eq!(census.color_pages, 1);
    assert_eq!(census.color_entries, 1);
    assert_eq!(census.staging_buffers, 0);
    assert_eq!(census.upload_submissions, 0);
    gpu.try_close(&mut resources)
        .unwrap_or_else(|_| panic!("settled test uploads must close"));
    assert!(resources.current().is_zero());
}

#[test]
pub(in crate::native::mechanics_adapter) fn alpha_and_color_physical_owner_merger_is_rejected() {
    assert!(physical_ownership_counts_match((1, 1), (1, 1), (1, 1)));
    assert!(!physical_ownership_counts_match((1, 1), (2, 0), (2, 0)));
    assert!(!physical_ownership_counts_match((1, 1), (1, 1), (2, 0)));
}

fn key() -> UiGlyphRasterKey {
    key_for_glyph_and_source(1, UiGlyphRasterSource::AlphaOutline)
}

fn key_for_glyph_and_source(glyph_id: u32, source: UiGlyphRasterSource) -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([4; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([5; 32], 0),
        glyph_id,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source,
        dpi_milli: 1_000,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}
