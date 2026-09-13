use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedAccessibilityProjection,
    UiMountedParticipationStatus, UiMountedProjectionView,
};

use super::super::UiHeadlessUnperformedEffect;

pub(super) fn unperformed_effects(
    projection: &UiMountedProjectionView,
    appearance_mechanic_count: u32,
) -> Result<Vec<UiHeadlessUnperformedEffect>, UiHostSurfacePresentationDenial> {
    let mut effects = vec![UiHeadlessUnperformedEffect::NativePaint {
        appearance_mechanic_count,
        portal_overlay_count: u32::try_from(projection.portal_overlays().rows().len())
            .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
        semantic_text_count: u32::try_from(projection.semantic_text().rows().len())
            .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
        preview_node_count: matching_node_count(projection, |node| {
            matches!(
                node.preview(),
                worth_ui_host_contract::UiMountedPreviewProjection::Resize { .. }
            )
        })?,
    }];
    effects.extend(node_unperformed_effects(projection)?);
    effects.extend(external_batch_effects(projection)?);
    Ok(effects)
}

fn node_unperformed_effects(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHeadlessUnperformedEffect>, UiHostSurfacePresentationDenial> {
    let mut effects = Vec::new();
    let accessibility = matching_node_count(projection, |node| {
        matches!(
            node.accessibility(),
            UiMountedAccessibilityProjection::Admitted(_)
        )
    })?;
    if accessibility > 0 {
        effects.push(UiHeadlessUnperformedEffect::Accessibility {
            node_count: accessibility,
        });
    }
    let diagnostic = matching_node_count(projection, |node| {
        matches!(
            node.diagnostic(),
            worth_ui_host_contract::UiMountedDiagnosticProjection::Reference(_)
                | worth_ui_host_contract::UiMountedDiagnosticProjection::IdentityOverlay(_)
        )
    })?;
    if diagnostic > 0 {
        effects.push(UiHeadlessUnperformedEffect::Diagnostic {
            node_count: diagnostic,
        });
    }
    Ok(effects)
}

fn external_batch_effects(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHeadlessUnperformedEffect>, UiHostSurfacePresentationDenial> {
    let mut effects = Vec::new();
    for (index, batch) in projection.spatial_batches().rows().iter().enumerate() {
        effects.push(UiHeadlessUnperformedEffect::CanvasSpatial {
            batch_index: u16::try_from(index)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            primitive_count: batch.primitive_count(),
            hit_region_count: batch.hit_region_count(),
            overlay_row_count: batch.overlay_row_count(),
            tool_state_row_count: batch.tool_state_row_count(),
        });
    }
    for (index, batch) in projection.realtime_batches().rows().iter().enumerate() {
        effects.push(UiHeadlessUnperformedEffect::Realtime {
            batch_index: u16::try_from(index)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?,
            overlay_row_count: batch.overlay_row_count(),
        });
    }
    Ok(effects)
}

fn matching_node_count(
    projection: &UiMountedProjectionView,
    predicate: impl Fn(&worth_ui_host_contract::UiMountedNodeProjectionView) -> bool,
) -> Result<u32, UiHostSurfacePresentationDenial> {
    u32::try_from(
        projection
            .nodes()
            .iter()
            .filter(|node| predicate(node))
            .count(),
    )
    .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)
}

pub(super) fn has_accessibility(projection: &UiMountedProjectionView) -> bool {
    projection.nodes().iter().any(|node| {
        matches!(
            node.accessibility(),
            UiMountedAccessibilityProjection::Admitted(_)
        )
    })
}

pub(super) fn has_focus(projection: &UiMountedProjectionView) -> bool {
    projection
        .nodes()
        .iter()
        .any(|node| node.participation().focus().status() == UiMountedParticipationStatus::Admitted)
}

pub(super) fn has_motion(projection: &UiMountedProjectionView) -> bool {
    projection.nodes().iter().any(|node| {
        matches!(
            node.motion(),
            worth_ui_host_contract::UiMountedMotionProjection::Admitted
        )
    })
}

pub(super) fn has_diagnostic(projection: &UiMountedProjectionView) -> bool {
    projection.nodes().iter().any(|node| {
        matches!(
            node.diagnostic(),
            worth_ui_host_contract::UiMountedDiagnosticProjection::Reference(_)
                | worth_ui_host_contract::UiMountedDiagnosticProjection::IdentityOverlay(_)
        )
    })
}
