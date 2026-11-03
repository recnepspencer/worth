//! Admit the one planned-state backing before consuming any planning input.

use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::{PlannedStateBacking, PlanningMemoryDenial, PlanningResidentAllowance};

use super::super::context::PlanningContext;
use super::super::resolved_basis::ResolvedPlanningState;
use super::execution_basis::ExecutionProducts;

pub(super) fn prepare(
    context: PlanningContext,
    basis: &ResolvedPlanningState,
    execution: &ExecutionProducts,
) -> Result<(PlanningContext, PlannedStateBacking, u64), PhysicalRecoveryOutcome> {
    let retained = super::super::resident_memory::live_bytes(&context, basis)
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<ExecutionProducts>() as u64))
        .and_then(|bytes| bytes.checked_add(execution.staging.owned_heap_bytes()?))
        .and_then(|bytes| bytes.checked_add(execution.publication.owned_heap_bytes()?))
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<PlanningResidentAllowance>() as u64)
        })
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<PlannedStateBacking>() as u64));
    let mut allowance = match PlanningResidentAllowance::new(
        retained.unwrap_or(u64::MAX),
        context.limits.recovery_memory_bytes,
    ) {
        Ok(allowance) if retained.is_some() => allowance,
        Ok(_) => {
            return Err(deny(
                context,
                basis,
                PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX },
                execution.planning_construction_peak,
            ))
        }
        Err(denial) => {
            return Err(deny(
                context,
                basis,
                denial,
                execution.planning_construction_peak,
            ))
        }
    };
    match PlannedStateBacking::prepare(
        &mut allowance,
        std::mem::size_of::<ResolvedPlanningState>() as u64,
    ) {
        Ok(backing) => Ok((context, backing, allowance.peak())),
        Err(denial) => Err(deny(
            context,
            basis,
            denial,
            execution.planning_construction_peak.max(allowance.peak()),
        )),
    }
}

fn deny(
    context: PlanningContext,
    basis: &ResolvedPlanningState,
    denial: PlanningMemoryDenial,
    measured_peak: u64,
) -> PhysicalRecoveryOutcome {
    match denial {
        PlanningMemoryDenial::RecoveryMemoryBytes { observed } => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            context.cost_denial_block(
                basis.planning_counters_at_peak(observed.max(measured_peak)),
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
            )
        }
        PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        } => context.block_with_planning_attempt_denial(
            PhysicalRecoveryBlockKind::RedoPlanning,
            basis.planning_counters_at_peak(measured_peak),
            "planned-state-allocation",
            None,
            PhysicalRecoveryPlanningDenial::PlannedStateAllocation {
                requested_bytes,
                cause,
            },
        ),
    }
}
