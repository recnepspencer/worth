use worth_store_recovery_physics::RecoveryPlanCost;

use crate::handoff::RecoveryOperationFateSet;
use crate::progression::PlannedPhysicalRecovery;

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningBasis;
use super::execution_basis::ExecutionProducts;

pub(super) fn construct(
    context: PlanningContext,
    basis: ResolvedPlanningBasis,
    execution: ExecutionProducts,
    plan_cost: RecoveryPlanCost,
    planning_counters: worth_store_recovery_physics::RecoveryPlanningCounters,
) -> PlannedPhysicalRecovery {
    PlannedPhysicalRecovery::new(
        context.authority,
        context.coordination,
        context.selection,
        basis.verified_selected_checkpoint_custody,
        basis.verified_selected_head_custody_v2,
        basis.verified_selected_no_release_custody,
        basis.verified_pending_wal_release_custody,
        basis.verified_ordered_historical_release_custody,
        basis.verified_effective_release_heads_v14,
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
    )
}
