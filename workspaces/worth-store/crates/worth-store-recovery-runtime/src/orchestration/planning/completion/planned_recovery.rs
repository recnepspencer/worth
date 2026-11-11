use worth_store_recovery_physics::RecoveryPlanCost;

use crate::handoff::RecoveryOperationFateSet;
use crate::progression::PlannedPhysicalRecovery;
use crate::progression::PlanningCustody;

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningState;
use crate::progression::ExecutionBasis;

pub(super) fn construct(
    context: PlanningContext,
    basis: ResolvedPlanningState,
    execution: ExecutionBasis,
    backing: crate::progression::PlannedStateBacking,
    plan_cost: RecoveryPlanCost,
    planning_counters: worth_store_recovery_physics::RecoveryPlanningCounters,
) -> Result<PlannedPhysicalRecovery, crate::entry::PhysicalRecoveryOutcome> {
    if matches!(&basis.custody, PlanningCustody::Unresolved) {
        return Err(context.block_with_planning_attempt_denial(
            crate::entry::PhysicalRecoveryBlockKind::SelectedCustody,
            planning_counters,
            "selected-custody-finalization",
            None,
            crate::entry::PhysicalRecoveryPlanningDenial::CustodyUnresolved,
        ));
    }
    let execution = execution.into_state();
    Ok(PlannedPhysicalRecovery::new(
        backing,
        context.authority,
        context.coordination,
        context.selection,
        basis.custody,
        basis.observed_pages.tier_custody,
        context.counters,
        context.root_protocol_denials,
        context.integrity,
        basis.sample,
        RecoveryOperationFateSet::new(
            basis.fates,
            basis
                .historical_consumed
                .expect("post-verification historical disposition is present"),
        ),
        basis.redo,
        plan_cost,
        planning_counters,
        context.root_protocol_counters,
        execution.staging,
        execution.publication,
        execution.quiescence,
        context.integrity_trace,
    ))
}
