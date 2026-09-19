use super::*;

pub(super) fn resolve_layer(
    projection: &UiMountedProjectionView,
    layer: UiMountedLayerProjection,
) -> Result<UiHeadlessLayerMechanic, UiHostSurfacePresentationDenial> {
    match layer {
        UiMountedLayerProjection::Omitted(reason) => Ok(UiHeadlessLayerMechanic::Omitted(reason)),
        UiMountedLayerProjection::Layer(reference) => {
            let row = projection
                .layers()
                .rows()
                .get(usize::from(reference.index()))
                .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?;
            Ok(UiHeadlessLayerMechanic::Ordered {
                semantic_order: row.semantic_order(),
                clip: resolve_clip(projection, row.clip())?,
            })
        }
    }
}

fn resolve_clip(
    projection: &UiMountedProjectionView,
    clip: UiMountedClipProjection,
) -> Result<UiHeadlessResolvedClip, UiHostSurfacePresentationDenial> {
    match clip {
        UiMountedClipProjection::Unclipped => Ok(UiHeadlessResolvedClip::Unclipped),
        UiMountedClipProjection::Omitted(reason) => Ok(UiHeadlessResolvedClip::Omitted(reason)),
        UiMountedClipProjection::Clip(reference) => projection
            .clips()
            .rows()
            .get(usize::from(reference.index()))
            .map(|_| UiHeadlessResolvedClip::Clip(reference.index()))
            .ok_or(UiHostSurfacePresentationDenial::MalformedProjection),
    }
}

pub(super) fn translate_nodes(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHeadlessNodeMechanic>, UiHostSurfacePresentationDenial> {
    projection
        .nodes()
        .iter()
        .map(|node| {
            let paint = match node.paint() {
                UiMountedPaintProjection::Omitted(reason) => {
                    UiHeadlessNodePaintMechanic::Omitted(reason)
                }
                UiMountedPaintProjection::CountOnlyBatch(reference) => {
                    projection
                        .paint_batches()
                        .rows()
                        .get(usize::from(reference.index()))
                        .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?;
                    UiHeadlessNodePaintMechanic::CountOnlyBatch(reference.index())
                }
            };
            Ok(UiHeadlessNodeMechanic::new(UiHeadlessNodeMechanicInput {
                mounted_instance: node.mounted_instance(),
                authored_position: node.authored_position(),
                role: node.role(),
                participation: node.participation(),
                allocation: node.allocation(),
                preview: node.preview(),
                paint,
                accessibility: node.accessibility(),
                motion: node.motion(),
                diagnostic: node.diagnostic(),
            }))
        })
        .collect()
}
