//! Rejoin one already-selected first V3 batch to the pending second V3 claim.
//! More historical batches remain fail-closed until the ordered fold exists.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, ReleasedDropWalFateWitnessV1,
    SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    EffectiveReleaseHeadDenial, VerifiedEffectiveReleaseHeadRosterV14,
    VerifiedOrderedHistoricalReleaseCustody, VerifiedOrderedRootEdge,
    VerifiedPendingWalReleaseCustody,
};

use super::{selected_controls::read_control, PlanningContext, ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::progression::PlanningCustody;

#[path = "ordered.rs"]
mod ordered;

pub(in crate::orchestration::planning::completion::blob_reclaim::selected_release_gate) fn admit_completed_history(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if basis.observed_pages.tier_custody.is_some()
        || !matches!(
            &basis.custody,
            PlanningCustody::Unresolved | PlanningCustody::SourceHeads(_)
        )
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let mut resident = match super::super::resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => return Err(context.redo_block(basis.planning_counters(), limit)),
    };
    let selected_head_v2 = match std::mem::replace(&mut basis.custody, PlanningCustody::Unresolved)
    {
        PlanningCustody::SourceHeads(base) => Some(base),
        PlanningCustody::Unresolved => None,
        _ => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let admitted = ordered::admit_roster(
        context,
        basis,
        1,
        None,
        selected_head_v2.as_ref(),
        &mut resident,
    );
    basis.custody = match selected_head_v2 {
        Some(base) => PlanningCustody::SourceHeads(base),
        None => PlanningCustody::Unresolved,
    };
    let (context, history, batches, head_replays) = admitted?;
    if !matches!(
        history.edges().last(),
        Some(VerifiedOrderedRootEdge::Ordinary(_))
    ) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    // Vec::into_boxed_slice may allocate a replacement while the admitted
    // Vec backing and all addressed control frames remain live.
    let replacement_headers = u64::try_from(batches.len())
        .ok()
        .and_then(|count| {
            count.checked_mul(std::mem::size_of::<
                worth_store_recovery_physics::VerifiedOrderedPendingWalReleaseBatch,
            >() as u64)
        })
        .unwrap_or(u64::MAX);
    let root_encoding = u64::from(context.authority.record_format.page_size().bytes())
        .checked_mul(2)
        .unwrap_or(u64::MAX);
    if resident
        .transient(replacement_headers.max(root_encoding))
        .is_err()
    {
        let limit = super::super::resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    let source_custody = std::mem::replace(&mut basis.custody, PlanningCustody::Unresolved);
    let claim = if let PlanningCustody::SourceHeads(base) = source_custody {
        VerifiedOrderedHistoricalReleaseCustody::admit_head_v2(
            &context.selection,
            &basis.observed_pages.selected_source.free_space,
            base,
            history,
            batches,
            context.limits.staging_bytes,
        )
    } else {
        let Some(shared) = context.coordination.owner().checkpoint() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        VerifiedOrderedHistoricalReleaseCustody::admit_no_release(
            &context.selection,
            shared.stream(),
            &basis.observed_pages.selected_source.free_space,
            history,
            batches,
            context.limits.staging_bytes,
        )
    };
    let Ok(claim) = claim else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let effective = VerifiedEffectiveReleaseHeadRosterV14::admit_ordered_completed(
        &claim,
        head_replays,
        context.limits.manifest_entries,
        context.limits.staging_bytes,
        resident.remaining(),
    );
    let effective = match effective {
        Ok(effective) => effective,
        Err(EffectiveReleaseHeadDenial::ResidentBoundExceeded { required, .. }) => {
            let _ = resident.transient(required);
            let limit = super::super::resident_basis::limit_failure(&context, &resident);
            return Err(context.redo_block(basis.planning_counters(), limit));
        }
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    if resident
        .bytes(effective.head_entries_heap_bytes().unwrap_or(u64::MAX))
        .is_err()
    {
        let limit = super::super::resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    basis.custody = PlanningCustody::OrderedCompleted {
        claim,
        effective_heads: effective,
    };
    Ok(context)
}

pub(super) fn attach(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    claim: &mut VerifiedPendingWalReleaseCustody,
    resident: &mut ResidentAllowance,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if basis.observed_pages.ordered_releases.is_some() {
        return ordered::attach(context, basis, claim, resident);
    }
    let mut advanced = basis
        .observed_pages
        .historical_drops
        .iter_mut()
        .filter(|evidence| evidence.chain.is_some());
    let Some(first) = advanced.next() else {
        return Ok(context);
    };
    if advanced.next().is_some()
        || !basis
            .verified_historical_release_operations
            .contains(&first.operation)
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let operation = first.operation;
    let descriptor_record = first.descriptor_record;
    let descriptor = first.descriptor;
    let manifest_bytes =
        u64::try_from(std::mem::size_of_val(first.manifest.dropped())).unwrap_or(u64::MAX);
    if resident.bytes(manifest_bytes).is_err() {
        let limit = super::super::resident_basis::limit_failure(&context, resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    let historical_manifest = first.manifest.clone();
    let Some(chain) = first.chain.take() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let mut members = basis
        .sample
        .wal_members()
        .iter()
        .filter(|member| member.operation_identity() == operation);
    let Some(member) = members.next() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if members.next().is_some()
        || member.lsn_range() != chain.first_lsn()
        || <[u8; 32]>::from(Sha256::digest(member.canonical_redo())) != chain.first_redo_sha256()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let frame_binding = {
        let selected_wal = context
            .integrity
            .admitted_wal()
            .recoverable_frame_iter(context.selection.wal_tail());
        let mut frames = selected_wal.filter(|frame| {
            frame.lsn_start() == member.lsn_range().start().get()
                && frame.lsn_end() == member.lsn_range().end_exclusive().get()
        });
        let binding = frames.next().map(|frame| {
            (
                frame.lsn_start(),
                frame.lsn_end(),
                frame.identity_digest(),
                frame.payload_digest(),
            )
        });
        (binding, frames.next().is_some())
    };
    let (Some((lsn_start, lsn_end, identity_digest, payload_digest)), false) = frame_binding else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let wal_fate = match ReleasedDropWalFateWitnessV1::new(
        lsn_start,
        lsn_end,
        identity_digest,
        payload_digest,
    ) {
        Ok(witness) => witness,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    let remaining_entries = basis.observed_pages.manifest_budget.remaining();
    let remaining_bytes = context
        .limits
        .observation_bytes
        .saturating_sub(context.counters.bytes_observed)
        .saturating_sub(basis.observed_pages.bytes_read)
        .saturating_sub(basis.observed_pages.candidate_bytes_read)
        .saturating_sub(basis.observed_pages.source_copy_bytes_read)
        .saturating_sub(basis.observed_pages.historical_publication_bytes_read);
    if remaining_entries == 0 || remaining_bytes == 0 {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("admitted nonzero historical control bounds");
    let format = context.authority.record_format;
    let mut scratch = 0;
    let descriptor_frame = read_control(
        &context.selection,
        &mut discovery,
        format,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
        &mut scratch,
        descriptor_record,
        BlobRecordKind::ReclaimDescriptorV3,
        resident,
    );
    let manifest_frame = read_control(
        &context.selection,
        &mut discovery,
        format,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
        &mut scratch,
        descriptor.base().manifest_record(),
        BlobRecordKind::DropSetManifestV3,
        resident,
    );
    let mut reservation = None;
    let mut valid = true;
    for route in context
        .selection
        .page_facts()
        .placements()
        .iter()
        .copied()
        .filter(|route| {
            route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::OriginalDropReserved)
        })
    {
        let Some(candidate) = read_control(
            &context.selection,
            &mut discovery,
            format,
            &mut basis.observed_pages.manifest_budget,
            &mut context.integrity_trace,
            &mut scratch,
            route.record(),
            BlobRecordKind::OriginalDropReserved,
            resident,
        ) else {
            valid = false;
            break;
        };
        let Ok(BlobRecordV1::OriginalDropReserved(value)) = decode_blob_record(candidate.bytes())
        else {
            valid = false;
            break;
        };
        if value.manifest_record() == descriptor.base().manifest_record()
            && value.request() == descriptor.custody().request()
        {
            if reservation.is_some() {
                valid = false;
                break;
            }
            reservation = Some(candidate);
        } else {
            let retained = u64::try_from(candidate.bytes().len()).unwrap_or(u64::MAX);
            drop(candidate);
            if resident.release(retained).is_err() {
                valid = false;
                break;
            }
        }
    }
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    basis.observed_pages.historical_publication_reads = basis
        .observed_pages
        .historical_publication_reads
        .saturating_add(counters.addressed_artifacts_read);
    basis.observed_pages.historical_publication_bytes_read = basis
        .observed_pages
        .historical_publication_bytes_read
        .saturating_add(counters.bytes_read);
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    let (Some(descriptor_frame), Some(manifest_frame), Some(reservation_frame)) =
        (descriptor_frame, manifest_frame, reservation)
    else {
        let limit = super::super::resident_basis::limit_failure(&context, resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    };
    let encoding_peak = u64::try_from(manifest_frame.bytes().len())
        .unwrap_or(u64::MAX)
        .checked_mul(3)
        .unwrap_or(u64::MAX)
        .checked_add(BLOB_CONTROL_FRAME_MAX_BYTES as u64)
        .unwrap_or(u64::MAX);
    if resident.transient(encoding_peak).is_err() {
        let limit = super::super::resident_basis::limit_failure(&context, resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    if !valid
        || descriptor_frame.bytes() != descriptor.encode()
        || manifest_frame.bytes() != historical_manifest.encode()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    if claim
        .attach_historical_first_batch(
            &context.selection,
            chain,
            &descriptor_frame,
            &reservation_frame,
            &manifest_frame,
            wal_fate,
            &basis.redo,
            &basis.fates,
            basis.sample.policy_identity(),
        )
        .is_err()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}
