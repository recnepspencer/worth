use crate::entry::{PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::RecoveryPublicationPlan;

use super::super::context::PlanningContext;

pub(super) fn admit(
    context: PlanningContext,
    planning_counters: worth_store_recovery_physics::RecoveryPlanningCounters,
    publication: &RecoveryPublicationPlan,
) -> Result<PlanningContext, PhysicalRecoveryOutcome> {
    let effects = RecoveryAllowance::declared(
        &context.limits,
        PhysicalRecoveryLimitDimension::PublicationEffects,
    );
    if let Some(limit) = effects.past(publication.expected_effects()) {
        return Err(context.redo_block(planning_counters, Some(limit)));
    }
    assert_eq!(
        context.effects_before,
        context.authority.media.recovery_effect_count()
    );
    Ok(context)
}
