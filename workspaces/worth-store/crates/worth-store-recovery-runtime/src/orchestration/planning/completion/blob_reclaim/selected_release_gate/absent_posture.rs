//! Admit the existing Absent selected-release custody family.
use super::{
    absent_posture_admits, selected_has_release, PlanningContext, PlanningCustody,
    ResolvedPlanningBasis,
};

pub(super) fn verify(
    context: PlanningContext,
    basis: &mut ResolvedPlanningBasis,
) -> Result<PlanningContext, crate::entry::PhysicalRecoveryOutcome> {
    let (next, has_release) = selected_has_release(context, basis)?;
    if !absent_posture_admits(has_release, &basis.custody) {
        return Err(next.redo_block(basis.planning_counters(), None));
    }
    basis.custody = PlanningCustody::NoCheckpoint;
    Ok(next)
}
