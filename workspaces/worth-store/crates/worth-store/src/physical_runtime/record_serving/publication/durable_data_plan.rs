use std::{collections::BTreeMap, num::NonZeroU64};

use worth_store_physical_format::{
    append_inline_records_owned, prepare_extent_chunk, ExtentChunkCoordinate, InlineRecordAppend,
    PersistedRecordIdentity,
};

use super::{
    append_observation::PublicationObservation, extent_publication::ExtentDataPlan,
    plan::CandidateDataArtifact, segment_publication::SegmentDataPlan,
};
use crate::physical_runtime::{
    durability::{
        CertifiedPriorPageBasis, PhysicalDataFrameIdentity, PreparedPhysicalDataFrame,
        PreparedPhysicalDataPlan,
    },
    record_serving::{
        planning::{prepared_payload::PreparedRecordPayloadPlan, PreparedPhysicalRootProjection},
        AdmittedPhysicalRecordFormat, RecordAppendDenial, RecordAppendError, RecordStreamFailure,
        RecordStreamFailureKind,
    },
};

pub(in crate::physical_runtime::record_serving) fn materialize_durable_data(
    mut payload: PreparedRecordPayloadPlan,
    format: AdmittedPhysicalRecordFormat,
    root_publication_allocation_bytes: NonZeroU64,
    manifest_capacity_transition: super::PhysicalManifestCapacityTransition,
) -> Result<(PreparedPhysicalDataPlan, PreparedPhysicalRootProjection), RecordAppendError> {
    let ordinals = payload
        .records
        .iter()
        .copied()
        .enumerate()
        .map(|(ordinal, record)| (record, ordinal as u32))
        .collect::<BTreeMap<_, _>>();
    let mut frames = Vec::new();
    for data in std::mem::take(&mut payload.data) {
        match data {
            CandidateDataArtifact::Segment(segment) => materialize_segment(
                segment,
                format,
                &ordinals,
                &mut frames,
                &mut payload.observation,
            )?,
            CandidateDataArtifact::Extent(extent) => materialize_extent(
                extent,
                format,
                &ordinals,
                &mut frames,
                &mut payload.observation,
            )?,
        }
    }
    let data = PreparedPhysicalDataPlan::new(frames, payload.records.len() as u32)
        .map_err(|_| invalid_plan())?;
    let root = PreparedPhysicalRootProjection {
        root_publication_allocation_bytes,
        source_root: payload.source_root,
        manifest_capacity_transition,
        placement: payload.placement,
        records: payload.records,
        payload_manifests: payload.payload_manifests,
        placements: payload.placements,
        segment_updates: payload.segment_updates,
        inline_allocations: payload.inline_allocations,
        last_inline_record: payload.last_inline_record,
        last_inline_segment: payload.last_inline_segment,
        observation: payload.observation,
    };
    Ok((data, root))
}

fn materialize_segment(
    segment: SegmentDataPlan,
    format: AdmittedPhysicalRecordFormat,
    ordinals: &BTreeMap<PersistedRecordIdentity, u32>,
    frames: &mut Vec<PreparedPhysicalDataFrame>,
    observation: &mut PublicationObservation,
) -> Result<(), RecordAppendError> {
    let page_bytes = u64::from(format.declaration().page_size().bytes());
    for (page_index, page) in segment.pages.into_iter().enumerate() {
        let prior = certify_inline_prior(page.existing_frame.as_ref())?;
        let appends = page
            .records
            .iter()
            .map(|(record, slot, bytes)| InlineRecordAppend::new(*record, *slot, bytes))
            .collect::<Vec<_>>();
        let (bytes, _) = append_inline_records_owned(
            format.declaration(),
            page.page,
            page.existing_frame.map(|image| image.into_bytes()),
            &appends,
        )
        .map_err(|_| invalid_plan())?;
        observation.observe_scratch(bytes.len());
        for (_, _, record_bytes) in &page.records {
            observation.observe_copy(record_bytes.len());
        }
        observation.observe_transfer(bytes.len());
        let offset = (page_index as u64)
            .checked_mul(page_bytes)
            .ok_or_else(invalid_plan)?;
        let length = u32::try_from(bytes.len()).map_err(|_| invalid_plan())?;
        let target =
            PhysicalDataFrameIdentity::inline_page(page.page, segment.artifact, offset, length)
                .ok_or_else(invalid_plan)?;
        let prior =
            prior.unwrap_or_else(|| CertifiedPriorPageBasis::for_unmaterialized_target(target));
        let mut redo_ordinals = page
            .records
            .iter()
            .map(|(record, _, _)| ordinals.get(record).copied().ok_or_else(invalid_plan))
            .collect::<Result<Vec<_>, _>>()?;
        redo_ordinals.sort_unstable();
        frames.push(
            PreparedPhysicalDataFrame::new(
                target,
                prior,
                redo_ordinals,
                bytes,
                format.declaration(),
            )
            .map_err(|_| invalid_plan())?,
        );
    }
    Ok(())
}

fn materialize_extent(
    mut extent: ExtentDataPlan,
    format: AdmittedPhysicalRecordFormat,
    ordinals: &BTreeMap<PersistedRecordIdentity, u32>,
    frames: &mut Vec<PreparedPhysicalDataFrame>,
    observation: &mut PublicationObservation,
) -> Result<(), RecordAppendError> {
    let redo_ordinal = ordinals
        .get(&extent.manifest.record())
        .copied()
        .ok_or_else(invalid_plan)?;
    let transfer = extent.manifest.chunk_payload_capacity() as usize;
    let mut completed = 0_u64;
    let mut artifact_offset = 0_u64;
    for ordinal in 1..=extent.manifest.chunk_count() {
        let expected =
            usize::try_from((extent.manifest.logical_bytes() - completed).min(transfer as u64))
                .map_err(|_| invalid_plan())?;
        let coordinate = ExtentChunkCoordinate::new(
            extent.manifest.record(),
            extent.manifest.extent_cell(),
            extent.manifest.logical_bytes(),
            completed,
            ordinal,
        )
        .ok_or_else(invalid_plan)?;
        let mut frame = prepare_extent_chunk(format.declaration(), coordinate, expected)
            .map_err(|_| invalid_plan())?;
        read_exact_source(
            &mut *extent.source,
            frame.payload_mut(),
            completed,
            observation,
        )?;
        let bytes = frame.seal();
        observation.observe_scratch(bytes.len());
        observation.observe_transfer(bytes.len());
        let length = u32::try_from(bytes.len()).map_err(|_| invalid_plan())?;
        let target = PhysicalDataFrameIdentity::extent_chunk(
            coordinate,
            extent.artifact,
            artifact_offset,
            length,
        )
        .ok_or_else(invalid_plan)?;
        frames.push(
            PreparedPhysicalDataFrame::new(
                target,
                CertifiedPriorPageBasis::for_unmaterialized_target(target),
                vec![redo_ordinal],
                bytes,
                format.declaration(),
            )
            .map_err(|_| invalid_plan())?,
        );
        completed = completed.saturating_add(expected as u64);
        artifact_offset = artifact_offset
            .checked_add(u64::from(length))
            .ok_or_else(invalid_plan)?;
    }
    reject_trailing_source(&mut *extent.source, completed)?;
    Ok(())
}

fn certify_inline_prior(
    image: Option<&super::ExistingDataFrameImage>,
) -> Result<Option<CertifiedPriorPageBasis>, RecordAppendError> {
    let Some(image) = image else {
        return Ok(None);
    };
    Ok(Some(
        CertifiedPriorPageBasis::for_integrity_admitted_materialized_source(
            image.admitted_prior_basis(),
        )
        .ok_or_else(invalid_plan)?,
    ))
}

fn read_exact_source(
    source: &mut dyn super::streaming::RecordWriteSource,
    target: &mut [u8],
    completed_before: u64,
    observation: &mut PublicationObservation,
) -> Result<(), RecordAppendError> {
    let mut filled = 0_usize;
    while filled < target.len() {
        let count = source.read_next(&mut target[filled..]).map_err(|_| {
            stream_failure(
                RecordStreamFailureKind::ProducerRejected,
                completed_before + filled as u64,
            )
        })?;
        if count == 0 {
            return Err(stream_failure(
                RecordStreamFailureKind::SourceEndedEarly,
                completed_before + filled as u64,
            ));
        }
        if count > target.len() - filled {
            return Err(stream_failure(
                RecordStreamFailureKind::InvalidTransferCount,
                completed_before + filled as u64,
            ));
        }
        observation.observe_copy(count);
        filled += count;
    }
    Ok(())
}

fn reject_trailing_source(
    source: &mut dyn super::streaming::RecordWriteSource,
    completed: u64,
) -> Result<(), RecordAppendError> {
    let mut extra = [0_u8; 1];
    let count = source
        .read_next(&mut extra)
        .map_err(|_| stream_failure(RecordStreamFailureKind::ProducerRejected, completed))?;
    if count == 0 {
        Ok(())
    } else {
        Err(stream_failure(
            RecordStreamFailureKind::SourceExceededDeclaredLength,
            completed,
        ))
    }
}

fn stream_failure(kind: RecordStreamFailureKind, completed: u64) -> RecordAppendError {
    RecordAppendError::StreamFailed(RecordStreamFailure::before_media_write(kind, completed))
}

fn invalid_plan() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)
}
