//! Complete already-selected release custody through the ordered C8 fold.

use super::ordered;
use super::{PlanningContext, ResolvedPlanningBasis};
use crate::entry::PhysicalRecoveryOrderedReleaseDenial as Denial;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;
use crate::progression::PlanningCustody;
use ordered::denial::block;
use worth_store_recovery_physics::{
    EffectiveReleaseHeadDenial, VerifiedEffectiveReleaseHeadRosterV14,
    VerifiedOrderedHistoricalReleaseCustody,
};

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
        return Err(block(context, basis, Denial::CustodyPosture, None));
    }
    let mut resident = match super::super::super::resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => {
            let cause = limit.map_or(Denial::ResidentBasis, |value| {
                Denial::ResidentBoundExceeded {
                    required: value.observed,
                    admitted: value.admitted,
                }
            });
            return Err(block(context, basis, cause, limit));
        }
    };
    let selected_head_v2 = match std::mem::replace(&mut basis.custody, PlanningCustody::Unresolved)
    {
        PlanningCustody::SourceHeads(base) => Some(base),
        PlanningCustody::Unresolved => None,
        _ => return Err(block(context, basis, Denial::CustodyPosture, None)),
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
        return Err(resident_block(context, basis, &resident));
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
            return Err(block(context, basis, Denial::MissingCheckpoint, None));
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
    let claim = match claim {
        Ok(claim) => claim,
        Err(cause) => return Err(block(context, basis, Denial::CompletedCustody(cause), None)),
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
        Err(cause @ EffectiveReleaseHeadDenial::ResidentBoundExceeded { required, .. }) => {
            let _ = resident.transient(required);
            let limit = super::super::super::resident_basis::limit_failure(&context, &resident);
            return Err(block(context, basis, Denial::EffectiveHeads(cause), limit));
        }
        Err(cause) => return Err(block(context, basis, Denial::EffectiveHeads(cause), None)),
    };
    if resident
        .bytes(effective.head_entries_heap_bytes().unwrap_or(u64::MAX))
        .is_err()
    {
        return Err(resident_block(context, basis, &resident));
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

fn resident_block(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    resident: &ResidentAllowance,
) -> crate::entry::PhysicalRecoveryOutcome {
    let limit = super::super::super::resident_basis::limit_failure(&context, resident);
    let cause = Denial::ResidentBoundExceeded {
        required: resident.exceeded_requirement().unwrap_or(u64::MAX),
        admitted: resident.used().saturating_add(resident.remaining()),
    };
    block(context, basis, cause, limit)
}
