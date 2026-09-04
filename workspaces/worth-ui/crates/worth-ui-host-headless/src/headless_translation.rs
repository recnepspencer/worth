use worth_ui_host_contract::{
    UiHostSurfacePresentationDenial, UiMountedClipProjection, UiMountedFrameConsumptionView,
    UiMountedLayerProjection, UiMountedPaintProjection, UiMountedProjectionView,
    UiMountedTableProjectionStatus,
};

use super::headless_transcript::{
    UiHeadlessMountedFrameTranscriptInput, UiHeadlessNodeMechanicInput,
    UiHeadlessTranscriptSuccessorIdentity,
};
use super::{
    UiHeadlessClipMechanic, UiHeadlessLayerMechanic, UiHeadlessMountedFrameTranscript,
    UiHeadlessNodeMechanic, UiHeadlessNodePaintMechanic, UiHeadlessPaintBatchMechanic,
    UiHeadlessRecorderCapacity, UiHeadlessResolvedClip, UiHeadlessResourceContact,
};

#[cfg(feature = "certification-support")]
pub(crate) mod appearance;
mod portal_overlay;
pub(super) mod semantic_text;
pub(super) mod static_paint;
mod unperformed_effects;

use unperformed_effects::{
    has_accessibility, has_diagnostic, has_focus, has_motion, unperformed_effects,
};

#[cfg(feature = "certification-support")]
pub(crate) fn translate_unpublished_appearance_work(
    work: &worth_ui_host_contract::UiMountedAppearanceWork,
) -> Result<
    super::headless_transcript::appearance::UiHeadlessAppearanceWorkTranscript,
    appearance::UiHeadlessAppearanceTranslationDenial,
> {
    appearance::translate(work)
}

#[cfg(feature = "certification-support")]
pub fn translate_unpublished_appearance_for_certification(
    projection: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
) -> Result<
    super::headless_transcript::appearance::UiHeadlessUnpublishedAppearanceFrameTranscript,
    appearance::UiHeadlessAppearanceTranslationDenial,
> {
    let fragments = projection
        .fragments()
        .iter()
        .map(|fragment| {
            let work = translate_unpublished_appearance_work(fragment.work())?;
            Ok(
                super::headless_transcript::appearance::UiHeadlessUnpublishedAppearanceFragmentTranscript::from_source(
                    fragment, work,
                ),
            )
        })
        .collect::<Result<Vec<_>, appearance::UiHeadlessAppearanceTranslationDenial>>()?;
    Ok(
        super::headless_transcript::appearance::UiHeadlessUnpublishedAppearanceFrameTranscript::from_source(
            projection, fragments,
        ),
    )
}

pub(super) fn translate_headless_frame(
    view: &UiMountedFrameConsumptionView<'_>,
    projection: &UiMountedProjectionView,
    capacity: UiHeadlessRecorderCapacity,
    mounted_order: &[worth_ui_host_contract::UiMountedPaintOrderIdentity],
    logical_damage: &[worth_ui_host_contract::UiMountedLogicalDamage],
) -> Result<UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
    static_paint::validate_protocol(view, projection)?;
    portal_overlay::validate(view, projection)?;
    validate_mechanic_capacity(projection, capacity)?;
    validate_external_batch_alignment(projection)?;
    let clips = translate_clips(projection)?;
    let filled_rects = static_paint::translate_filled_rects(projection)?;
    let portal_overlays = projection.portal_overlays().rows().to_vec();
    let semantic_text = semantic_text::translate(view, projection)?;
    let mut paint_batches = translate_paint_batches(projection)?;
    paint_batches.sort_by_key(paint_order);
    let nodes = translate_nodes(projection)?;
    let unperformed_effects = unperformed_effects(projection)?;
    Ok(UiHeadlessMountedFrameTranscript::new(
        UiHeadlessMountedFrameTranscriptInput {
            host_session_identity: view.host_session_identity(),
            protocol: view.protocol(),
            attempt: view.attempt(),
            frame: projection.frame(),
            binding: view.requirement().binding(),
            nodes,
            clips,
            filled_rects,
            portal_overlays,
            semantic_text,
            paint_batches,
            paint_order: mounted_order.to_vec(),
            logical_damage: logical_damage.to_vec(),
            unperformed_effects,
        },
    ))
}

pub(super) fn translate_auxiliary_delta(
    identity: UiHeadlessTranscriptSuccessorIdentity,
    projection: &UiMountedProjectionView,
    retained: &UiHeadlessMountedFrameTranscript,
    capacity: UiHeadlessRecorderCapacity,
    mounted_order: &[worth_ui_host_contract::UiMountedPaintOrderIdentity],
    logical_damage: &[worth_ui_host_contract::UiMountedLogicalDamage],
) -> Result<UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
    validate_mechanic_capacity(projection, capacity)?;
    validate_external_batch_alignment(projection)?;
    let clips = translate_clips(projection)?;
    let mut paint_batches = translate_paint_batches(projection)?;
    paint_batches.sort_by_key(paint_order);
    Ok(UiHeadlessMountedFrameTranscript::new(
        UiHeadlessMountedFrameTranscriptInput {
            host_session_identity: identity.host_session_identity,
            protocol: identity.protocol,
            attempt: identity.attempt,
            frame: identity.frame,
            binding: identity.binding,
            nodes: translate_nodes(projection)?,
            clips,
            filled_rects: retained.filled_rects().to_vec(),
            portal_overlays: retained.portal_overlays().to_vec(),
            semantic_text: retained.semantic_text().to_vec(),
            paint_batches,
            paint_order: mounted_order.to_vec(),
            logical_damage: logical_damage.to_vec(),
            unperformed_effects: unperformed_effects(projection)?,
        },
    ))
}

fn validate_external_batch_alignment(
    projection: &UiMountedProjectionView,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let canvas = projection.paint_batches().rows().iter().filter(|row| {
        row.primitive_kind()
            == worth_ui_host_contract::UiMountedPaintPrimitiveKind::CanvasSpatialBatch
    });
    let realtime = projection.paint_batches().rows().iter().filter(|row| {
        row.primitive_kind() == worth_ui_host_contract::UiMountedPaintPrimitiveKind::RealtimeBatch
    });
    let canvas_count = canvas.clone().count();
    let realtime_count = realtime.clone().count();
    let canvas_aligned = canvas
        .zip(projection.spatial_batches().rows())
        .all(|(paint, spatial)| paint.primitive_count() == spatial.primitive_count());
    let realtime_aligned = realtime
        .zip(projection.realtime_batches().rows())
        .all(|(paint, overlay)| paint.primitive_count() == u32::from(overlay.overlay_row_count()));
    if canvas_count != projection.spatial_batches().rows().len()
        || realtime_count != projection.realtime_batches().rows().len()
        || !canvas_aligned
        || !realtime_aligned
    {
        Err(UiHostSurfacePresentationDenial::MalformedProjection)
    } else {
        Ok(())
    }
}

fn validate_mechanic_capacity(
    projection: &UiMountedProjectionView,
    capacity: UiHeadlessRecorderCapacity,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let count = [
        projection.nodes().len(),
        projection.clips().rows().len(),
        projection.filled_rects().rows().len(),
        projection.portal_overlays().rows().len(),
        projection.semantic_text().rows().len(),
        projection.paint_batches().rows().len(),
        projection.spatial_batches().rows().len(),
        projection.realtime_batches().rows().len(),
        1,
        usize::from(has_accessibility(projection)),
        usize::from(has_focus(projection)),
        usize::from(has_motion(projection)),
        usize::from(has_diagnostic(projection)),
    ]
    .into_iter()
    .try_fold(0usize, usize::checked_add)
    .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)?;
    if count > capacity.mechanics_per_frame() {
        Err(UiHostSurfacePresentationDenial::CapacityExceeded)
    } else {
        Ok(())
    }
}

fn translate_clips(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHeadlessClipMechanic>, UiHostSurfacePresentationDenial> {
    if matches!(
        projection.clips().status(),
        UiMountedTableProjectionStatus::Omitted(_)
    ) && !projection.clips().rows().is_empty()
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    projection
        .clips()
        .rows()
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let parent = row.parent().map(|reference| reference.index());
            if parent.is_some_and(|parent| usize::from(parent) >= index) {
                return Err(UiHostSurfacePresentationDenial::MalformedProjection);
            }
            Ok(UiHeadlessClipMechanic::new(row.bounds(), parent))
        })
        .collect()
}

fn translate_paint_batches(
    projection: &UiMountedProjectionView,
) -> Result<Vec<UiHeadlessPaintBatchMechanic>, UiHostSurfacePresentationDenial> {
    projection
        .paint_batches()
        .rows()
        .iter()
        .enumerate()
        .map(|(index, batch)| {
            let batch_index = u16::try_from(index)
                .map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)?;
            let layer = resolve_layer(projection, batch.layer())?;
            let resource = match batch.resource() {
                Some(reference) => {
                    let entry = projection
                        .resources()
                        .resolve(reference)
                        .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?;
                    Some(UiHeadlessResourceContact::new(
                        entry.content_identity(),
                        entry.kind(),
                        entry.byte_len(),
                    ))
                }
                None => None,
            };
            Ok(UiHeadlessPaintBatchMechanic::new(
                batch_index,
                batch.primitive_kind(),
                batch.primitive_count(),
                layer,
                resource,
            ))
        })
        .collect()
}

fn resolve_layer(
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

fn translate_nodes(
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
                UiMountedPaintProjection::FilledRect(reference) => {
                    projection
                        .filled_rects()
                        .resolve(reference)
                        .ok_or(UiHostSurfacePresentationDenial::MalformedProjection)?;
                    UiHeadlessNodePaintMechanic::FilledRect(reference.index())
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

fn paint_order(batch: &UiHeadlessPaintBatchMechanic) -> (u8, u32, u16) {
    match batch.layer() {
        UiHeadlessLayerMechanic::Ordered { semantic_order, .. } => {
            (0, semantic_order, batch.batch_index())
        }
        UiHeadlessLayerMechanic::Omitted(_) => (1, u32::MAX, batch.batch_index()),
    }
}
