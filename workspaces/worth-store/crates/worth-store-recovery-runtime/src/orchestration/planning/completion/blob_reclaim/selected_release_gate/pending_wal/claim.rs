//! Final pending V14 claim construction and optional historical release join.

use worth_store_recovery_physics::{
    VerifiedPendingWalReleaseCustody, VerifiedSelectedReleaseHeadReplayV14,
    WitnessedSelectedControlFrame,
};

use super::super::resident_basis;
use super::{historical, member_fate::PendingMemberFate, PlanningContext, ResolvedPlanningBasis};
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    projection_index: usize,
    member: PendingMemberFate,
    manifest: WitnessedSelectedControlFrame,
    reservation: WitnessedSelectedControlFrame,
    head_replay: VerifiedSelectedReleaseHeadReplayV14,
    mut resident: ResidentAllowance,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let clone_bytes = head_replay.owned_heap_bytes().unwrap_or(u64::MAX);
    let assembly = u64::try_from(std::mem::size_of::<
        worth_store_recovery_physics::VerifiedSelectedReleaseHeadCustodyV2,
    >())
    .unwrap_or(u64::MAX);
    if resident
        .bytes(clone_bytes)
        .and_then(|_| resident.bytes(assembly))
        .is_err()
    {
        let limit = resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    // Admission canonically encodes the source root and descriptor and decodes
    // the drop manifest. Include conversion backing, not just wire length.
    let controls_decode = u64::try_from(manifest.bytes().len())
        .unwrap_or(u64::MAX)
        .checked_mul(3)
        .unwrap_or(u64::MAX)
        .checked_add(worth_store_physical_format::BLOB_CONTROL_FRAME_MAX_BYTES as u64)
        .unwrap_or(u64::MAX)
        .max(
            u64::from(context.authority.record_format.page_size().bytes())
                .checked_mul(2)
                .unwrap_or(u64::MAX),
        );
    if resident.transient(controls_decode).is_err() {
        let limit = resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    }
    let projection = &basis.redo.projections()[projection_index];
    let admitted = match basis.verified_selected_head_custody_v2.take() {
        Some(selected_head) => VerifiedPendingWalReleaseCustody::admit_from_head_v2(
            &context.selection,
            selected_head,
            head_replay.clone(),
            basis.observed_pages.tier_custody.as_ref(),
            projection,
            &basis.redo,
            &reservation,
            &manifest,
            member.wal_fate,
            member.canonical_redo_digest,
            &basis.fates,
            basis.sample.policy_identity(),
        ),
        None => VerifiedPendingWalReleaseCustody::admit_with_head_replay(
            &context.selection,
            head_replay.clone(),
            basis.observed_pages.tier_custody.as_ref(),
            projection,
            &basis.redo,
            &reservation,
            &manifest,
            member.wal_fate,
            member.canonical_redo_digest,
            &basis.fates,
            basis.sample.policy_identity(),
        ),
    };
    let mut claim = match admitted {
        Ok(claim) => claim,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    context = historical::attach(context, basis, &mut claim, &mut resident)?;
    // Reopen must consume these mandatory head copies, not allocate them
    // after root publication. Preparation is storage, never published custody.
    if basis.verified_effective_release_heads_v14.is_none() && claim.ordered_history().is_none() {
        let source_count = claim
            .selected_head_v2()
            .map_or(0, |base| base.selected_heads().len()) as u64;
        let head_backing = source_count
            .checked_mul(2)
            .and_then(|entries| entries.checked_add(1))
            .and_then(|entries| {
                entries.checked_mul(std::mem::size_of::<
                    worth_store_physical_format::ReleaseCustodyHeadEntryV1,
                >() as u64)
            })
            .unwrap_or(u64::MAX);
        if resident.transient(head_backing).is_err() {
            let limit = resident_basis::limit_failure(&context, &resident);
            return Err(context.redo_block(basis.planning_counters(), limit));
        }
        let prepared =
            claim.prepare_effective_heads(context.limits.manifest_entries, resident.remaining());
        let heap_bytes = match prepared {
            Ok(bytes) => bytes,
            Err(
                worth_store_recovery_physics::EffectiveReleaseHeadDenial::ResidentBoundExceeded {
                    required,
                    ..
                },
            ) => {
                let _ = resident.transient(required);
                let limit = resident_basis::limit_failure(&context, &resident);
                return Err(context.redo_block(basis.planning_counters(), limit));
            }
            Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
        };
        if resident.bytes(heap_bytes).is_err() {
            let limit = resident_basis::limit_failure(&context, &resident);
            return Err(context.redo_block(basis.planning_counters(), limit));
        }
    }
    basis
        .observed_pages
        .historical_publication_peak_scratch_bytes = basis
        .observed_pages
        .historical_publication_peak_scratch_bytes
        .max(resident.peak());
    basis.verified_pending_wal_release_custody = Some(claim);
    basis.verified_pending_release_head_replay = Some(head_replay);
    Ok(context)
}
