//! Attach every completed V3 edge under one checkpoint before the final
//! pending V3. Controls are addressed to each edge's own candidate root.

use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_store_physical_format::ReleasedDropWalFateWitnessV1;
use worth_store_recovery_physics::{
    EffectiveReleaseHeadDenial, VerifiedAddressedCheckpointReleaseBase,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedOrderedRootEdge, VerifiedPendingWalReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2,
};

use super::{PlanningContext, ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

fn limit_block(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    resident: &ResidentAllowance,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = super::super::resident_basis::limit_failure(&context, resident);
    context.redo_block(basis.planning_counters(), limit)
}

fn reserve_roster<T>(
    values: &mut Vec<T>,
    count: usize,
    resident: &mut ResidentAllowance,
) -> Result<(), ()> {
    let requested = u64::try_from(count)
        .ok()
        .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
        .unwrap_or(u64::MAX);
    resident.transient(requested).map_err(|_| ())?;
    values.try_reserve_exact(count).map_err(|_| ())?;
    let actual = u64::try_from(values.capacity())
        .ok()
        .and_then(|capacity| capacity.checked_mul(std::mem::size_of::<T>() as u64))
        .unwrap_or(u64::MAX);
    resident.bytes(actual).map_err(|_| ())
}

pub(super) fn attach(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    claim: &mut VerifiedPendingWalReleaseCustody,
    resident: &mut ResidentAllowance,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let (context, history, batches, head_replays) = admit_roster(
        context,
        basis,
        1,
        claim.addressed_release_base(),
        claim.selected_head_v2(),
        resident,
    )?;
    let replacement_headers = u64::try_from(batches.len())
        .ok()
        .and_then(|count| {
            count.checked_mul(std::mem::size_of::<VerifiedOrderedPendingWalReleaseBatch>() as u64)
        })
        .unwrap_or(u64::MAX);
    if resident.transient(replacement_headers).is_err() {
        return Err(limit_block(context, basis, resident));
    }
    if claim
        .attach_ordered_history(history, batches, context.limits.staging_bytes)
        .is_err()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let prepared = claim.prepare_ordered_effective_heads(
        head_replays,
        context.limits.manifest_entries,
        context.limits.staging_bytes,
        resident.remaining(),
    );
    let backing = match prepared {
        Ok(backing) => backing,
        Err(EffectiveReleaseHeadDenial::ResidentBoundExceeded { required, .. }) => {
            let _ = resident.transient(required);
            return Err(limit_block(context, basis, resident));
        }
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    if resident.bytes(backing).is_err() {
        return Err(limit_block(context, basis, resident));
    }
    Ok(context)
}

pub(super) fn admit_roster(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    minimum_releases: usize,
    selected_base: Option<&VerifiedAddressedCheckpointReleaseBase>,
    selected_head_v2: Option<&VerifiedSelectedReleaseHeadCustodyV2>,
    resident: &mut ResidentAllowance,
) -> Result<
    (
        PlanningContext,
        Arc<worth_store_recovery_physics::VerifiedOrderedRootHistory>,
        Vec<VerifiedOrderedPendingWalReleaseBatch>,
        Vec<(usize, VerifiedOrderedReleasedHeadReplayV14)>,
    ),
    crate::entry::PhysicalRecoveryOutcome,
> {
    let Some(releases) = basis.observed_pages.ordered_releases.take() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let histories = basis
        .observed_pages
        .historical_drops
        .iter()
        .filter_map(|evidence| evidence.ordered_history.as_ref());
    let mut histories = histories.peekable();
    let Some(history) = histories.next().cloned() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    if releases.len() < minimum_releases
        || histories.any(|other| !Arc::ptr_eq(other, &history))
        || releases.len() != basis.observed_pages.historical_drops.len()
    {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let roster_bytes = (releases.len() as u64)
        .checked_mul(std::mem::size_of::<VerifiedOrderedPendingWalReleaseBatch>() as u64)
        .and_then(|value| {
            value.checked_add(
                (releases.len() as u64)
                    .checked_mul(
                        std::mem::size_of::<(usize, VerifiedOrderedReleasedHeadReplayV14)>() as u64,
                    )?,
            )
        })
        .and_then(|value| value.checked_add(history.peak_scratch_bytes()));
    let Some(roster_bytes) = roster_bytes else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let Some(mut remaining_bytes) = context.limits.staging_bytes.checked_sub(roster_bytes) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let mut batches = Vec::new();
    let mut head_replays = Vec::new();
    if reserve_roster(&mut batches, releases.len(), resident).is_err() {
        return Err(limit_block(context, basis, resident));
    }
    if reserve_roster(&mut head_replays, releases.len(), resident).is_err() {
        return Err(limit_block(context, basis, resident));
    }
    for release in releases {
        if !basis
            .verified_historical_release_operations
            .contains(&release.operation)
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let mut edges = history.edges().iter().enumerate().filter(|(_, edge)| {
            matches!(edge, VerifiedOrderedRootEdge::Released(value)
                if value.operation() == release.operation)
        });
        let Some((edge_index, VerifiedOrderedRootEdge::Released(edge))) = edges.next() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if edges.next().is_some()
            || edge.descriptor_record() != release.descriptor_frame.record()
            || edge.descriptor_frame_sha256() != release.descriptor_frame.payload_sha256()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let mut members = basis
            .sample
            .wal_members()
            .iter()
            .filter(|member| member.operation_identity() == release.operation);
        let Some(member) = members.next() else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        if members.next().is_some()
            || member.lsn_range() != edge.lsn()
            || <[u8; 32]>::from(Sha256::digest(member.canonical_redo())) != edge.redo_sha256()
        {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        let frame_binding = {
            let mut frames = context
                .integrity
                .admitted_wal()
                .recoverable_frame_iter(context.selection.wal_tail())
                .filter(|frame| {
                    frame.lsn_start() == edge.lsn().start().get()
                        && frame.lsn_end() == edge.lsn().end_exclusive().get()
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
        let (Some((lsn_start, lsn_end, identity_digest, payload_digest)), false) = frame_binding
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let Ok(wal_fate) =
            ReleasedDropWalFateWitnessV1::new(lsn_start, lsn_end, identity_digest, payload_digest)
        else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        let encoding_peak = u64::try_from(release.manifest_frame.bytes().len())
            .ok()
            .and_then(|len| len.checked_mul(3))
            .and_then(|bytes| {
                bytes.checked_add(worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES as u64)
            })
            .unwrap_or(u64::MAX);
        let decoded_manifest =
            u64::try_from(std::mem::size_of_val(release.manifest.dropped())).unwrap_or(u64::MAX);
        let decode_window = encoding_peak
            .checked_add(decoded_manifest)
            .unwrap_or(u64::MAX);
        if resident.transient(decode_window).is_err() || resident.bytes(decoded_manifest).is_err() {
            return Err(limit_block(context, basis, resident));
        }
        let admitted = VerifiedOrderedPendingWalReleaseBatch::admit(
            &history,
            edge_index,
            release.descriptor_frame,
            release.reservation_frame,
            release.manifest_frame,
            wal_fate,
            &basis.fates,
            basis.sample.policy_identity(),
            context
                .selection
                .checkpoint()
                .expect("ordered history has checkpoint")
                .checkpoint()
                .compaction_cutover()
                .wal_cutoff_lsn_exclusive(),
            selected_base,
            selected_head_v2,
            &batches,
            remaining_bytes,
        );
        let Ok(batch) = admitted else {
            return Err(context.redo_block(basis.planning_counters(), None));
        };
        remaining_bytes = remaining_bytes.saturating_sub(batch.retained_bytes());
        batches.push(batch);
        head_replays.push((edge_index, release.head_replay));
    }
    Ok((context, history, batches, head_replays))
}
