//! First release batch result proof after the publication has disappeared.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV2, BlobReclaimDescriptorV3, BlobRecordKind,
    CurrentPhysicalRecordPlacement, DropSetManifestV3, PersistedRecordIdentity,
    SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::{historical_publication, historical_publication::HistoricalFailure, record};

pub(super) fn verify_initial_result(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV2,
    manifest: &DropSetManifestV3,
    descriptor_record: PersistedRecordIdentity,
    source_count: u64,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let format = context.authority.record_format;
    let encoded_descriptor = descriptor.encode();
    let descriptor_digest: [u8; 32] = Sha256::digest(&encoded_descriptor).into();
    let (next, result_count) = historical_publication::observe(
        context,
        basis,
        descriptor.candidate_root_generation(),
        descriptor_record,
        |discovery, root, route, budget, trace, scratch| {
            if !matches!(route,
                Some(CurrentPhysicalRecordPlacement::Extent(extent))
                    if extent.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV2))
            {
                return Err(HistoricalFailure::Invalid);
            }
            let bytes = record::read(
                discovery,
                format,
                route,
                descriptor_record,
                BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                budget,
                trace,
                scratch,
            )?;
            if bytes != encoded_descriptor
                || <[u8; 32]>::from(Sha256::digest(&bytes)) != descriptor_digest
            {
                return Err(HistoricalFailure::Invalid);
            }
            Ok(root.record_count())
        },
    )?;
    context = next;
    if source_count
        .checked_sub(u64::from(manifest.count()))
        .and_then(|count| count.checked_add(1))
        != Some(result_count)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, retained_manifest) = historical_publication::observe(
        context,
        basis,
        descriptor.candidate_root_generation(),
        descriptor.manifest_record(),
        |discovery, _, route, budget, trace, scratch| {
            if !matches!(route,
                Some(CurrentPhysicalRecordPlacement::Extent(extent))
                    if extent.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV3))
            {
                return Err(HistoricalFailure::Invalid);
            }
            let bytes = record::read(
                discovery,
                format,
                route,
                descriptor.manifest_record(),
                BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                budget,
                trace,
                scratch,
            )?;
            Ok(<[u8; 32]>::from(Sha256::digest(bytes)) == descriptor.manifest_frame_sha256())
        },
    )?;
    context = next;
    if !retained_manifest {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, publication_absent) = historical_publication::observe(
        context,
        basis,
        descriptor.candidate_root_generation(),
        manifest.dropped()[0],
        |_, _, route, _, _, _| Ok(route.is_none()),
    )?;
    context = next;
    if !publication_absent {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}

pub(super) fn verify_v3_result(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV3,
    descriptor_record: PersistedRecordIdentity,
    manifest: &DropSetManifestV3,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let base = descriptor.base();
    let encoded = descriptor.encode();
    let format = context.authority.record_format;
    let (next, result_count) = historical_publication::observe(
        context,
        basis,
        base.candidate_root_generation(),
        descriptor_record,
        |discovery, root, route, budget, trace, scratch| {
            if !matches!(route,
                Some(CurrentPhysicalRecordPlacement::Extent(extent))
                    if extent.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3))
            {
                return Err(HistoricalFailure::Invalid);
            }
            let bytes = record::read(
                discovery,
                format,
                route,
                descriptor_record,
                BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                budget,
                trace,
                scratch,
            )?;
            if bytes != encoded {
                return Err(HistoricalFailure::Invalid);
            }
            Ok(root.record_count())
        },
    )?;
    context = next;
    let (next, source_count) = historical_publication::observe(
        context,
        basis,
        base.source_root_generation(),
        base.manifest_record(),
        |_, root, route, _, _, _| {
            route
                .map(|_| root.record_count())
                .ok_or(HistoricalFailure::Invalid)
        },
    )?;
    context = next;
    if source_count
        .checked_sub(u64::from(manifest.count()))
        .and_then(|count| count.checked_add(1))
        != Some(result_count)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let (next, retained) = historical_publication::observe(
        context,
        basis,
        base.candidate_root_generation(),
        base.manifest_record(),
        |discovery, _, route, budget, trace, scratch| {
            if !matches!(route,
                Some(CurrentPhysicalRecordPlacement::Extent(extent))
                    if extent.content_class()
                        == SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV3))
            {
                return Err(HistoricalFailure::Invalid);
            }
            let bytes = record::read(
                discovery,
                format,
                route,
                base.manifest_record(),
                BLOB_CONTROL_FRAME_MAX_BYTES as u64,
                budget,
                trace,
                scratch,
            )?;
            Ok(<[u8; 32]>::from(Sha256::digest(&bytes)) == base.manifest_frame_sha256())
        },
    )?;
    context = next;
    if !retained {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    for dropped in manifest.dropped() {
        let (next, absent) = historical_publication::observe(
            context,
            basis,
            base.candidate_root_generation(),
            *dropped,
            |_, _, route, _, _, _| Ok(route.is_none()),
        )?;
        context = next;
        if !absent {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    Ok(context)
}
