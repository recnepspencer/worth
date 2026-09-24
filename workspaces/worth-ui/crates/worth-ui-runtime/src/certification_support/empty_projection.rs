use worth_ui_host_contract::{
    UiMountedContentGeneration, UiMountedFrameIdentity, UiMountedPaintBatchTable,
    UiMountedProjectionView, UiMountedProjectionViewInput, UiMountedRealtimeBatchTable,
    UiMountedResourceTable, UiMountedSemanticTextTable, UiMountedSpatialBatchTable,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
};

pub fn empty_projection_for_certification() -> UiMountedProjectionView {
    empty_projection_at_for_certification(
        UiMountedFrameIdentity::mint_unbound().expect("frame identity"),
        UiSemanticSurfaceIdentity::mint_unbound().expect("surface identity"),
        UiSurfaceBindingGeneration::mint_unbound().expect("binding generation"),
    )
}

/// An empty projection of `frame` on one surface binding.
pub fn empty_projection_at_for_certification(
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
) -> UiMountedProjectionView {
    UiMountedProjectionView::new(UiMountedProjectionViewInput {
        frame,
        surface,
        binding,
        content_generation: UiMountedContentGeneration::mint_unbound().expect("content generation"),
        nodes: Vec::new(),
        clips: worth_ui_host_contract::UiMountedClipTable::produced(Vec::new()),
        layers: worth_ui_host_contract::UiMountedLayerTable::produced(Vec::new()),
        portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
        semantic_text: UiMountedSemanticTextTable::empty(),
        hit_tests: worth_ui_host_contract::UiMountedHitTestTable::empty(),
        paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
        spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
        realtime_batches: UiMountedRealtimeBatchTable::new(Vec::new()),
        resources: UiMountedResourceTable::new(Vec::new()),
        authored_paint_commands: Vec::new(),
        authored_paint_order: Vec::new(),
    })
}
