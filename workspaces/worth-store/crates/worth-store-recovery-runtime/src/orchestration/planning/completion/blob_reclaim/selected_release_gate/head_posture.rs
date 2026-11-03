//! Admit the existing HeadV2 selected-release custody family.
use super::{
    head_v2, pending_replay_matches, pending_wal, PlanningContext, PlanningCustody,
    ResolvedPlanningBasis,
};

pub(super) fn verify(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    if let PlanningCustody::PendingPrepared { claim: pending, .. } = &basis.custody {
        let admitted = pending.selected_head_v2().is_some_and(|custody| {
            custody.selected_root() == context.selection.root().selected().manifest()
        }) && pending_replay_matches(basis);
        return if admitted {
            Ok(context)
        } else {
            Err(context.redo_block(basis.planning_counters(), None))
        };
    }
    if matches!(&basis.custody, PlanningCustody::Unresolved) {
        context = head_v2::admit(context, basis)?;
    }
    if basis.observed_pages.ordered_releases.is_some() {
        return pending_wal::admit_completed_history(context, basis);
    }
    let PlanningCustody::SourceHeads(custody) = &basis.custody else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let selected = context.selection.root().selected().manifest();
    if custody.checkpoint_source_root().release_custody_head_root()
        != selected.release_custody_head_root()
        || custody
            .checkpoint_source_root()
            .next_release_custody_head_block()
            != selected.next_release_custody_head_block()
    {
        // A newer selected head root requires its complete ordered V14
        // history, not a blessing of checkpoint-source entries.
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    Ok(context)
}
