use crate::{
    UiMountedEffectFamily, UiMountedNodeProjectionView, UiMountedPaintBatchTable,
    UiMountedPaintPrimitiveKind, UiMountedRealtimeBatchTable, UiMountedSpatialBatchTable,
};

pub(super) fn derive(
    nodes: &[UiMountedNodeProjectionView],
    paint_batches: &UiMountedPaintBatchTable,
    spatial_batches: &UiMountedSpatialBatchTable,
    realtime_batches: &UiMountedRealtimeBatchTable,
) -> Vec<UiMountedEffectFamily> {
    let mut effects = Vec::new();
    push_if(
        &mut effects,
        has_canvas(paint_batches, spatial_batches),
        UiMountedEffectFamily::CanvasSpatial,
    );
    push_if(
        &mut effects,
        has_realtime(paint_batches, realtime_batches),
        UiMountedEffectFamily::Realtime,
    );
    push_if(
        &mut effects,
        nodes.iter().any(has_native_paint),
        UiMountedEffectFamily::NativePaint,
    );
    push_if(
        &mut effects,
        nodes.iter().any(has_accessibility),
        UiMountedEffectFamily::Accessibility,
    );
    push_if(
        &mut effects,
        nodes.iter().any(has_diagnostic),
        UiMountedEffectFamily::Diagnostic,
    );
    push_if(
        &mut effects,
        nodes.iter().any(has_identity_overlay),
        UiMountedEffectFamily::IdentityOverlay,
    );
    effects
}

fn push_if(effects: &mut Vec<UiMountedEffectFamily>, present: bool, family: UiMountedEffectFamily) {
    if present {
        effects.push(family);
    }
}

fn has_canvas(paint: &UiMountedPaintBatchTable, spatial: &UiMountedSpatialBatchTable) -> bool {
    !spatial.rows().is_empty()
        || paint
            .rows()
            .iter()
            .any(|row| row.primitive_kind() == UiMountedPaintPrimitiveKind::CanvasSpatialBatch)
}

fn has_realtime(paint: &UiMountedPaintBatchTable, realtime: &UiMountedRealtimeBatchTable) -> bool {
    !realtime.rows().is_empty()
        || paint
            .rows()
            .iter()
            .any(|row| row.primitive_kind() == UiMountedPaintPrimitiveKind::RealtimeBatch)
}

fn has_native_paint(node: &UiMountedNodeProjectionView) -> bool {
    !node.semantic_text().is_empty()
        || matches!(
            node.preview(),
            crate::UiMountedPreviewProjection::Resize { .. }
        )
}

fn has_accessibility(node: &UiMountedNodeProjectionView) -> bool {
    matches!(
        node.accessibility(),
        crate::UiMountedAccessibilityProjection::Admitted(_)
    )
}

fn has_diagnostic(node: &UiMountedNodeProjectionView) -> bool {
    matches!(
        node.diagnostic(),
        crate::UiMountedDiagnosticProjection::Reference(_)
    )
}

fn has_identity_overlay(node: &UiMountedNodeProjectionView) -> bool {
    matches!(
        node.diagnostic(),
        crate::UiMountedDiagnosticProjection::IdentityOverlay(_)
    )
}
