//! Admit one post-checkpoint V3 drop from the actual C.9 WAL suffix. The
//! selected NoRelease marker remains checkpoint-source truth only.

use super::super::super::super::{context::PlanningContext, resolved_basis::ResolvedPlanningBasis};
use super::resident_basis;
use crate::progression::PlanningCustody;

#[path = "pending_wal/claim.rs"]
mod claim;
#[path = "pending_wal/directory.rs"]
mod directory;
#[path = "pending_wal/head_replay.rs"]
mod head_replay;
#[path = "pending_wal/historical.rs"]
mod historical;
#[path = "pending_wal/member_fate.rs"]
mod member_fate;
#[path = "pending_wal/observation.rs"]
mod observation;
#[path = "pending_wal/selected_controls.rs"]
mod selected_controls;

pub(super) fn admit(
    mut context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
    projection_index: usize,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    // V14 release authority is either the independently joined V2 source
    // roster or a genuine NoRelease marker. A headless released checkpoint
    // cannot enter through the old addressed/global-tip admissions.
    if !matches!(
        &basis.custody,
        PlanningCustody::Unresolved | PlanningCustody::SourceHeads(_)
    ) {
        return Err(context.redo_block(basis.planning_counters(), None));
    }
    let mut resident = match resident_basis::seed(&context, basis) {
        Ok(resident) => resident,
        Err(limit) => return Err(context.redo_block(basis.planning_counters(), limit)),
    };
    let projection = &basis.redo.projections()[projection_index];
    let Some(member) = member_fate::witness(&context, basis, projection) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let Some((remaining_entries, remaining_bytes)) = observation::bounds(&context, basis) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    let mut discovery = context
        .authority
        .media
        .bounded_discovery(remaining_entries, remaining_bytes)
        .expect("positive pending-WAL control inspection bounds");
    let format = context.authority.record_format;
    let mut scratch = 0;
    let controls = selected_controls::observe(
        &context.selection,
        &mut discovery,
        format,
        &mut basis.observed_pages.manifest_budget,
        &mut context.integrity_trace,
        &mut scratch,
        member.descriptor,
        &mut resident,
    );
    let reservation_count = controls
        .as_ref()
        .map_or(0, |controls| controls.reservation_count());
    let witnesses = controls.and_then(|controls| controls.into_witnesses());
    // A WAL path is only a claim until the actual selected head namespace
    // supplies the exact source frames. Keep its proof separate from the
    // checkpoint-source roster; it advances the effective post-WAL root.
    let head_replay = if witnesses.is_some() {
        head_replay::admit(
            &mut discovery,
            &mut basis.observed_pages.manifest_budget,
            projection,
            &basis.redo,
            &context.selection,
            context.limits.staging_bytes,
            &mut resident,
        )
    } else {
        None
    };
    let directory_replacement = directory::admit(
        &context.selection,
        basis,
        projection_index,
        format,
        &mut context.integrity_trace,
        &mut discovery,
        &mut resident,
    );
    let counters = discovery.counters();
    context.authority.media = discovery.finish();
    observation::record_selected_reads(
        basis,
        counters.addressed_artifacts_read,
        counters.bytes_read,
        resident.peak(),
        reservation_count,
    );
    let Some((manifest, reservation)) = witnesses else {
        let limit = resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    };
    let Some(head_replay) = head_replay else {
        let limit = resident_basis::limit_failure(&context, &resident);
        return Err(context.redo_block(basis.planning_counters(), limit));
    };
    let directory_replacement = match directory_replacement {
        Ok(proof) => proof,
        Err(denial) => {
            let limit = resident_basis::limit_failure(&context, &resident);
            return Err(context.block_with_planning_attempt_denial(
                crate::entry::PhysicalRecoveryBlockKind::RedoPlanning,
                basis
                    .planning_counters()
                    .with_peak_recovery_bytes(resident.peak()),
                "released-directory-replacement",
                limit,
                denial,
            ));
        }
    };
    let Some(replay) = crate::progression::PendingReleaseReplay::new(
        &basis.redo.projections()[projection_index],
        head_replay,
        directory_replacement,
    ) else {
        return Err(context.redo_block(basis.planning_counters(), None));
    };
    claim::admit(
        context,
        basis,
        projection_index,
        member,
        manifest,
        reservation,
        replay,
        resident,
    )
}

pub(super) use historical::admit_completed_history;
