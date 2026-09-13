use std::collections::BTreeMap;

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableInlineRecordPlacement, PhysicalRecordFormatDeclaration, RecordArtifactFile,
    RecordFrameCoordinate,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, RecoveryPageObservation,
};

use super::{super::PageObservationFailure, observed::required_observed};
use crate::progression::RecoverySelectedSegmentPage;

struct InlinePageObservationPlan {
    selected: RecoverySelectedSegmentPage,
    artifact: RecordArtifactFile,
    offset: u64,
    page_bytes: u32,
}

/// Choose an exact selected image anchor; physics proves historical supersession.
pub(crate) fn selected_inline_target<'a>(
    placement: DurableInlineRecordPlacement,
    targets: &[&'a PhysicalRedoTarget],
    format: PhysicalRecordFormatDeclaration,
    entries: &BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
) -> &'a PhysicalRedoTarget {
    entries
        .get(&(placement.segment().get(), placement.page().get()))
        .and_then(|selected| {
            targets.iter().copied().find(|target| {
                target.identity()
                    == PhysicalRedoTargetIdentity::InlinePage {
                        segment: placement.segment().get(),
                        page: placement.page().get(),
                        generation: placement.page_generation(),
                    }
                    && target.artifact()
                        == RecordArtifactFile::Segment {
                            segment: placement.segment().get(),
                            generation: selected.entry.data_generation(),
                        }
                    && target.artifact_offset()
                        == u64::from(selected.entry.frame_index())
                            * u64::from(format.page_size().bytes())
                    && target.artifact_length() == format.page_size().bytes()
            })
        })
        .unwrap_or(targets[0])
}

pub(crate) fn observe_inline(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    placement: DurableInlineRecordPlacement,
    target: &PhysicalRedoTarget,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    entries: &BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<RecoveryPageObservation, PageObservationFailure> {
    let plan = plan_inline_observation(placement, target, format, entries)?;
    let page = required_observed(
        discovery.read_segment_range(
            placement.segment().get(),
            plan.selected.entry.data_generation(),
            plan.offset,
            plan.page_bytes,
            byte_limit,
        ),
        Some(target.identity()),
        plan.artifact,
    )?;
    let range = PhysicalByteRange::new(plan.offset, u64::from(plan.page_bytes))
        .map_err(|_| PageObservationFailure::InvalidPage(target.identity()))?;
    let scope = PhysicalArtifactScope::inline_page(
        discovery.store_identity(),
        format,
        placement.page_cell(),
        range,
    );
    let projection =
        crate::integrity_ingress::admit_page_projection(&page, scope, plan.artifact, integrity)
            .map_err(|_| PageObservationFailure::InvalidPage(target.identity()))?;
    let source = RecordFrameCoordinate::new(plan.artifact, plan.offset, plan.page_bytes)
        .ok_or(PageObservationFailure::InvalidPage(target.identity()))?;
    Ok(RecoveryPageObservation::materialized(
        PhysicalRedoTargetIdentity::InlinePage {
            segment: placement.segment().get(),
            page: placement.page().get(),
            generation: placement.page_generation(),
        },
        projection.page_lsn.get(),
        projection.encoded_digest,
        source,
        plan.selected.routing_identity,
    ))
}

fn plan_inline_observation(
    placement: DurableInlineRecordPlacement,
    target: &PhysicalRedoTarget,
    format: PhysicalRecordFormatDeclaration,
    entries: &BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
) -> Result<InlinePageObservationPlan, PageObservationFailure> {
    let resolved = entries
        .get(&(placement.segment().get(), placement.page().get()))
        .copied()
        .filter(|resolved| {
            let entry = resolved.entry;
            entry.page_cell() == placement.page_cell()
                && entry.page_generation() == placement.page_generation()
                && entry.data_page_count() <= placement.segment_page_capacity()
        })
        .ok_or(PageObservationFailure::InvalidManifest {
            target: Some(target.identity()),
            artifact: entries
                .get(&(placement.segment().get(), placement.page().get()))
                .map_or(
                    RecordArtifactFile::RootManifest {
                        generation: placement.page_generation(),
                    },
                    |resolved| resolved.membership_artifact,
                ),
        })?;
    let entry = resolved.entry;
    let page_bytes = format.page_size().bytes();
    let offset = u64::from(entry.frame_index())
        .checked_mul(u64::from(page_bytes))
        .ok_or(PageObservationFailure::InvalidPage(target.identity()))?;
    require_matching_target(
        placement,
        target,
        entry.data_generation(),
        offset,
        page_bytes,
    )?;
    let artifact = RecordArtifactFile::Segment {
        segment: placement.segment().get(),
        generation: entry.data_generation(),
    };
    Ok(InlinePageObservationPlan {
        selected: resolved,
        artifact,
        offset,
        page_bytes,
    })
}

fn require_matching_target(
    placement: DurableInlineRecordPlacement,
    target: &PhysicalRedoTarget,
    data_generation: u64,
    offset: u64,
    page_bytes: u32,
) -> Result<(), PageObservationFailure> {
    let PhysicalRedoTargetIdentity::InlinePage { generation, .. } = target.identity() else {
        return Err(PageObservationFailure::InvalidTarget(target.identity()));
    };
    let RecordArtifactFile::Segment {
        segment,
        generation: target_data_generation,
    } = target.artifact()
    else {
        return Err(PageObservationFailure::InvalidTarget(target.identity()));
    };
    let materialized = target_data_generation == data_generation
        && generation == placement.page_generation()
        && target.artifact_offset() == offset;
    let successor = data_generation.checked_add(1) == Some(target_data_generation)
        && placement.page_generation().checked_add(1) == Some(generation);
    if target.artifact_length() == page_bytes
        && segment == placement.segment().get()
        && (materialized || successor)
    {
        Ok(())
    } else {
        Err(PageObservationFailure::InvalidTarget(target.identity()))
    }
}
