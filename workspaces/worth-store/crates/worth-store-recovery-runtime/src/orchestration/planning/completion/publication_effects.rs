use crate::entry::{
    PhysicalRecoveryLimitDimension, PhysicalRecoveryLimitFailure, PhysicalRecoveryOutcome,
};
use crate::progression::RecoveryPublicationPlan;

use super::super::context::PlanningContext;

pub(super) fn admit(
    context: PlanningContext,
    planning_counters: worth_store_recovery_physics::RecoveryPlanningCounters,
    publication: &RecoveryPublicationPlan,
) -> Result<PlanningContext, PhysicalRecoveryOutcome> {
    if publication.expected_effects() > context.limits.publication_effects {
        let admitted = context.limits.publication_effects;
        return Err(context.redo_block(
            planning_counters,
            Some(PhysicalRecoveryLimitFailure {
                dimension: PhysicalRecoveryLimitDimension::PublicationEffects,
                observed: publication.expected_effects(),
                admitted,
            }),
        ));
    }
    assert_eq!(
        context.effects_before,
        context.authority.media.recovery_effect_count()
    );
    Ok(context)
}
