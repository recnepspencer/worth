//! Admit the existing NoRelease selected-release custody family.
use super::{
    pending_replay_matches, pending_wal, selected_has_release, PlanningContext, PlanningCustody,
    ResolvedPlanningState,
};

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningState,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if let PlanningCustody::PendingPrepared { claim: pending, .. } = &basis.custody {
        if pending.marker().is_none() || !pending_replay_matches(basis) {
            return Err(context.redo_block(basis.planning_counters(), None));
        }
        return Ok(context);
    }
    if basis.observed_pages.ordered_releases.is_some() {
        return pending_wal::admit_completed_history(context, basis);
    }
    let (next, has_release) = selected_has_release(context, basis)?;
    context = next;
    if has_release {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let Some(shared) = context.coordination.owner().checkpoint() else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let claim = match worth_store_recovery_physics::VerifiedSelectedNoReleaseCustody::claim_selected_no_release(
        &context.selection,
        shared.stream(),
    ) {
        Ok(claim) => claim,
        Err(_) => return Err(context.redo_block(basis.planning_counters(), None)),
    };
    if !matches!(&basis.custody, PlanningCustody::Unresolved) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    basis.custody = PlanningCustody::NoRelease(claim);
    Ok(context)
}
