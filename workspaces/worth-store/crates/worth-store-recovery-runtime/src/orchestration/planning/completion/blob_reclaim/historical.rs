//! Exact historical result-root proof for a drop already made durable.
//! Dropped source media may have been retired; generation alone is insufficient.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobReclaimDescriptorV1, DropSetManifestV1, PersistedRecordIdentity,
    BLOB_CONTROL_FRAME_MAX_BYTES,
};

use super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::super::historical_publication::{self, HistoricalFailure};
use super::record;

pub(super) fn verify_result(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    descriptor: BlobReclaimDescriptorV1,
    manifest: &DropSetManifestV1,
    descriptor_record: PersistedRecordIdentity,
    descriptor_digest: [u8; 32],
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let format = context.authority.record_format;
    let (next, source_count) = historical_publication::observe(
        context,
        basis,
        descriptor.source_root_generation(),
        descriptor.manifest_record(),
        |_, root, route, _, _, _| {
            if route.is_none() {
                Err(HistoricalFailure::Invalid)
            } else {
                Ok(root.record_count())
            }
        },
    )?;
    context = next;
    let (next, result_count) = historical_publication::observe(
        context,
        basis,
        descriptor.candidate_root_generation(),
        descriptor_record,
        |discovery, root, route, budget, trace, scratch| {
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
            if <[u8; 32]>::from(Sha256::digest(&bytes)) != descriptor_digest
                || bytes != descriptor.encode()
            {
                return Err(HistoricalFailure::Invalid);
            }
            Ok(root.record_count())
        },
    )?;
    context = next;
    if source_count
        .checked_sub(u64::from(manifest.count()))
        .and_then(|remaining| remaining.checked_add(1))
        != Some(result_count)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    for (identity, digest, maximum) in [
        (
            descriptor.manifest_record(),
            descriptor.manifest_frame_sha256(),
            BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        ),
        (
            manifest.source_basis().declaration_record(),
            manifest.source_basis().declaration_frame_sha256(),
            156,
        ),
        (
            manifest.source_basis().abandoned_record(),
            manifest.source_basis().abandoned_frame_sha256(),
            145,
        ),
    ] {
        let (next, retained) = historical_publication::observe(
            context,
            basis,
            descriptor.candidate_root_generation(),
            identity,
            |discovery, _, route, budget, trace, scratch| {
                let bytes = record::read(
                    discovery, format, route, identity, maximum, budget, trace, scratch,
                )?;
                Ok(<[u8; 32]>::from(Sha256::digest(bytes)) == digest)
            },
        )?;
        context = next;
        if !retained {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    for identity in manifest.dropped() {
        let (next, absent) = historical_publication::observe(
            context,
            basis,
            descriptor.candidate_root_generation(),
            *identity,
            |_, _, route, _, _, _| Ok(route.is_none()),
        )?;
        context = next;
        if !absent {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
    }
    Ok(context)
}
