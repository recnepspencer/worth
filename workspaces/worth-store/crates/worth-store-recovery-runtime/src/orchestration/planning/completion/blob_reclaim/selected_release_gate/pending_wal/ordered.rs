//! Admit every ordered V3 edge under one checkpoint. Controls are addressed
//! to each edge's own candidate root.

use std::sync::Arc;

use worth_store_recovery_physics::{
    EffectiveReleaseHeadDenial, PhysicsBound, VerifiedAddressedCheckpointReleaseBase,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedOrderedReleasedHeadReplayV14,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedReleaseHeadCustodyV2,
};

use super::{PlanningContext, ResolvedPlanningBasis};
use crate::entry::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryOrderedReleaseDenial as Denial,
    PhysicalRecoveryOrderedReleaseStorage as Storage,
};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::orchestration::recovery_budget::RecoveryAllowance;

#[path = "ordered/denial.rs"]
pub(super) mod denial;
#[path = "ordered/member_binding.rs"]
mod member_binding;
#[path = "ordered/roster_storage.rs"]
mod roster_storage;
use roster_storage::reserve_roster;

fn resident_block(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    resident: &ResidentAllowance,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = super::super::resident_basis::limit_failure(&context, resident);
    denial::block(context, basis, Denial::ResidentBoundExceeded, limit)
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
        return Err(resident_block(context, basis, resident));
    }
    if let Err(cause) = claim.attach_ordered_history(history, batches, context.limits.staging_bytes)
    {
        return Err(denial::block(
            context,
            basis,
            Denial::PendingAttachment(cause),
            None,
        ));
    }
    let prepared = claim.prepare_ordered_effective_heads(
        head_replays,
        context.limits.manifest_entries,
        context.limits.staging_bytes,
        resident.remaining(),
    );
    let backing = match prepared {
        Ok(backing) => backing,
        Err(cause @ EffectiveReleaseHeadDenial::Limit(past))
            if past.dimension() == PhysicsBound::ResidentBytes =>
        {
            let _ = resident.transient(past.observed());
            let limit = super::super::resident_basis::limit_failure(&context, resident);
            return Err(denial::block(
                context,
                basis,
                Denial::EffectiveHeads(cause),
                limit,
            ));
        }
        Err(cause) => {
            return Err(denial::block(
                context,
                basis,
                Denial::EffectiveHeads(cause),
                None,
            ))
        }
    };
    if resident.bytes(backing).is_err() {
        return Err(resident_block(context, basis, resident));
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
        return Err(denial::block(context, basis, Denial::MissingReleases, None));
    };
    let histories = basis
        .observed_pages
        .historical_drops
        .iter()
        .map(|evidence| &evidence.ordered_history);
    let mut histories = histories.peekable();
    let Some(history) = histories.next().map(Arc::clone) else {
        return Err(denial::block(context, basis, Denial::MissingHistory, None));
    };
    if releases.len() < minimum_releases
        || histories.any(|other| !Arc::ptr_eq(other, &history))
        || releases.len() != basis.observed_pages.historical_drops.len()
    {
        return Err(denial::block(context, basis, Denial::RosterBinding, None));
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
    let staging = RecoveryAllowance::declared(
        &context.limits,
        PhysicalRecoveryLimitDimension::StagingBytes,
    );
    // A count past every count is no limit.
    let Some(required) = roster_bytes else {
        return Err(denial::block(
            context,
            basis,
            Denial::StagingBoundExceeded,
            None,
        ));
    };
    let mut remaining_bytes = match staging.admit(required) {
        Ok(required) => staging.admitted() - required,
        Err(limit) => {
            return Err(denial::block(
                context,
                basis,
                Denial::StagingBoundExceeded,
                Some(limit.into()),
            ))
        }
    };
    let mut batches = Vec::new();
    let mut head_replays = Vec::new();
    if let Err(cause) = reserve_roster(&mut batches, releases.len(), Storage::BatchRoster, resident)
    {
        let limit = super::super::resident_basis::limit_failure(&context, resident);
        return Err(denial::block(context, basis, cause, limit));
    }
    if let Err(cause) = reserve_roster(
        &mut head_replays,
        releases.len(),
        Storage::HeadReplayRoster,
        resident,
    ) {
        let limit = super::super::resident_basis::limit_failure(&context, resident);
        return Err(denial::block(context, basis, cause, limit));
    }
    for release in releases {
        let member = match member_binding::join(&context, basis, &history, &release) {
            Ok(member) => member,
            Err(cause) => return Err(denial::block(context, basis, cause, None)),
        };
        let edge_index = member.edge_index();
        let wal_fate = member.wal_fate();
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
            return Err(resident_block(context, basis, resident));
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
        let batch = match admitted {
            Ok(batch) => batch,
            Err(cause) => {
                let (cause, limit) = denial::batch(release.operation, edge_index, cause, staging);
                return Err(denial::block(context, basis, cause, limit));
            }
        };
        remaining_bytes = remaining_bytes.saturating_sub(batch.retained_bytes());
        batches.push(batch);
        head_replays.push((edge_index, release.head_replay));
    }
    Ok((context, history, batches, head_replays))
}
