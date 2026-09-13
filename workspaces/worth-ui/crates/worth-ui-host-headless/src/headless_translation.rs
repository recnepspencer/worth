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

pub(crate) mod appearance;
mod nodes;
mod portal_overlay;
pub(super) mod semantic_text;
mod unperformed_effects;

use nodes::{resolve_layer, translate_nodes};

use unperformed_effects::{
    has_accessibility, has_diagnostic, has_focus, has_motion, unperformed_effects,
};

#[cfg(test)]
pub(crate) fn translate_appearance_fragment_work(
    source: &worth_ui_host_contract::UiMountedAppearanceWork,
) -> Result<
    super::headless_transcript::appearance::UiHeadlessAppearanceWorkTranscript,
    appearance::UiHeadlessAppearanceTranslationDenial,
> {
    appearance::translate(source)
}

pub(crate) fn translate_appearance_work(
    source: &worth_ui_host_contract::UiMountedAppearancePresentationWork,
) -> Result<
    super::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript,
    appearance::UiHeadlessAppearanceTranslationDenial,
> {
    let fragments = source
        .fragments()
        .iter()
        .map(|fragment| {
            appearance::translate(fragment.work()).map(|work| {
                super::headless_transcript::appearance::UiHeadlessAppearanceFragmentTranscript::from_source(
                    fragment, work,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(super::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript::from_source(source, fragments))
}

pub(super) fn translate_view_appearance(
    view: &UiMountedFrameConsumptionView<'_>,
) -> Result<
    Option<super::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript>,
    UiHostSurfacePresentationDenial,
> {
    let Some(source) = view.appearance_work() else {
        return Ok(None);
    };
    if source.frame() != view.frame()
        || source.presentation() != view.attempt()
        || source.requirement() != view.requirement()
    {
        return Err(UiHostSurfacePresentationDenial::MalformedProjection);
    }
    translate_appearance_work(source)
        .map(Some)
        .map_err(|_| UiHostSurfacePresentationDenial::MalformedProjection)
}

#[cfg(feature = "certification-support")]
pub fn translate_appearance_projection_for_certification(
    projection: &worth_ui_host_contract::UiUnpublishedAppearanceFrameProjection,
) -> Result<
    super::headless_transcript::appearance::UiHeadlessAppearanceProjectionTranscript,
    appearance::UiHeadlessAppearanceTranslationDenial,
> {
    let fragments = projection
        .fragments()
        .iter()
        .map(|fragment| {
            appearance::translate(fragment.work()).map(|work| {
                super::headless_transcript::appearance::UiHeadlessAppearanceFragmentTranscript::from_source(
                    fragment, work,
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(super::headless_transcript::appearance::UiHeadlessAppearanceProjectionTranscript::from_source(projection, fragments))
}

pub(super) fn translate_headless_frame(
    view: &UiMountedFrameConsumptionView<'_>,
    projection: &UiMountedProjectionView,
    capacity: UiHeadlessRecorderCapacity,
    mounted_order: &[worth_ui_host_contract::UiMountedPaintOrderIdentity],
    logical_damage: &[worth_ui_host_contract::UiMountedLogicalDamage],
) -> Result<UiHeadlessMountedFrameTranscript, UiHostSurfacePresentationDenial> {
    portal_overlay::validate(view, projection)?;
    validate_mechanic_capacity(projection, view.appearance_work(), capacity)?;
    validate_external_batch_alignment(projection)?;
    let clips = translate_clips(projection)?;
    let appearance_work = translate_view_appearance(view)?;
    let portal_overlays = projection.portal_overlays().rows().to_vec();
    let semantic_text = semantic_text::translate(view, projection)?;
    let mut paint_batches = translate_paint_batches(projection)?;
    paint_batches.sort_by_key(paint_order);
    let nodes = translate_nodes(projection)?;
    let unperformed_effects = unperformed_effects(
        projection,
        appearance_mechanic_count(view.appearance_work())?,
    )?;
    Ok(UiHeadlessMountedFrameTranscript::new(
        UiHeadlessMountedFrameTranscriptInput {
            host_session_identity: view.host_session_identity(),
            protocol: view.protocol(),
            attempt: view.attempt(),
            frame: projection.frame(),
            binding: view.requirement().binding(),
            nodes,
            clips,
            appearance_work,
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
    let appearance_work = retained.appearance_work().cloned();
    validate_translated_appearance_capacity(
        appearance_work.as_ref(),
        base_row_count(projection)?,
        capacity,
    )?;
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
            appearance_work,
            portal_overlays: retained.portal_overlays().to_vec(),
            semantic_text: retained.semantic_text().to_vec(),
            paint_batches,
            paint_order: mounted_order.to_vec(),
            logical_damage: logical_damage.to_vec(),
            unperformed_effects: unperformed_effects(
                projection,
                retained
                    .appearance_work()
                    .map_or(Ok(0), |appearance| appearance.mechanic_count())?,
            )?,
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
    appearance: Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork>,
    capacity: UiHeadlessRecorderCapacity,
) -> Result<(), UiHostSurfacePresentationDenial> {
    validate_appearance_capacity(appearance, base_row_count(projection)?, capacity)
}

pub(super) fn base_row_count(
    projection: &UiMountedProjectionView,
) -> Result<usize, UiHostSurfacePresentationDenial> {
    [
        projection.nodes().len(),
        projection.clips().rows().len(),
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
    .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)
}

pub(super) fn validate_appearance_capacity(
    appearance: Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork>,
    base_count: usize,
    capacity: UiHeadlessRecorderCapacity,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let count = base_count
        .checked_add(appearance_row_count(appearance)?)
        .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)?;
    if count > capacity.mechanics_per_frame() {
        Err(UiHostSurfacePresentationDenial::CapacityExceeded)
    } else {
        Ok(())
    }
}

fn validate_translated_appearance_capacity(
    appearance: Option<
        &super::headless_transcript::appearance::UiHeadlessAppearancePresentationTranscript,
    >,
    base_count: usize,
    capacity: UiHeadlessRecorderCapacity,
) -> Result<(), UiHostSurfacePresentationDenial> {
    let appearance_count = appearance.map_or(Ok(0), |work| work.row_count())?;
    let count = base_count
        .checked_add(appearance_count)
        .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)?;
    if count > capacity.mechanics_per_frame() {
        Err(UiHostSurfacePresentationDenial::CapacityExceeded)
    } else {
        Ok(())
    }
}

pub(super) fn appearance_row_count(
    appearance: Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork>,
) -> Result<usize, UiHostSurfacePresentationDenial> {
    appearance.into_iter().try_fold(0usize, |total, work| {
        let total = total
            .checked_add(work.fragments().len())
            .and_then(|value| value.checked_add(work.sample_overrides().len()))
            .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)?;
        work.fragments().iter().try_fold(total, |total, fragment| {
            total
                .checked_add(fragment.work().successor().mechanics().len())
                .and_then(|value| value.checked_add(fragment.work().changes().len()))
                .and_then(|value| value.checked_add(fragment.work().damage().len()))
                .and_then(|value| value.checked_add(fragment.text_candidates().len()))
                .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)
        })
    })
}

fn appearance_mechanic_count(
    appearance: Option<&worth_ui_host_contract::UiMountedAppearancePresentationWork>,
) -> Result<u32, UiHostSurfacePresentationDenial> {
    let count = appearance
        .into_iter()
        .flat_map(|work| work.fragments())
        .map(|fragment| fragment.work().successor().mechanics().len())
        .try_fold(0usize, usize::checked_add)
        .ok_or(UiHostSurfacePresentationDenial::CapacityExceeded)?;
    u32::try_from(count).map_err(|_| UiHostSurfacePresentationDenial::CapacityExceeded)
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

fn paint_order(batch: &UiHeadlessPaintBatchMechanic) -> (u8, u32, u16) {
    match batch.layer() {
        UiHeadlessLayerMechanic::Ordered { semantic_order, .. } => {
            (0, semantic_order, batch.batch_index())
        }
        UiHeadlessLayerMechanic::Omitted(_) => (1, u32::MAX, batch.batch_index()),
    }
}
