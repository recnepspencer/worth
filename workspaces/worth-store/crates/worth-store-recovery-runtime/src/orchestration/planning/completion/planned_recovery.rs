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
        context.counters,
        context.root_protocol_denials,
        context.integrity,
        basis.sample,
        RecoveryOperationFateSet::new(basis.fates),
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
