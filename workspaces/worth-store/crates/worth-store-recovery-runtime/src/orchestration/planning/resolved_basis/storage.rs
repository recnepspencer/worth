//! Checked ownership of the actual resolved planning state.
use super::super::context::PlanningContext;
use super::state::ResolvedPlanningState;
use crate::entry::{
    PhysicalRecoveryBlockKind, PhysicalRecoveryLimitDimension, PhysicalRecoveryOutcome,
    PhysicalRecoveryPlanningDenial,
};
use crate::orchestration::recovery_budget::RecoveryAllowance;
use crate::progression::{PlanningMemoryDenial, PlanningResidentAllowance};

pub(in crate::orchestration::planning) struct ResolvedPlanningBasis {
    values: Vec<ResolvedPlanningState>,
}

pub(super) struct ResolvedStateBackingDenial {
    cause: PlanningMemoryDenial,
    peak: u64,
}

impl ResolvedPlanningBasis {
    pub(super) fn capture(
        context: &PlanningContext,
        mut state: ResolvedPlanningState,
    ) -> Result<Self, ResolvedStateBackingDenial> {
        let overflow = || PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX };
        let retained = super::super::resident_memory::live_bytes(context, &state)
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<PlanningResidentAllowance>() as u64)
            })
            .ok_or_else(|| ResolvedStateBackingDenial {
                cause: overflow(),
                peak: state.capture_peak,
            })?;
        let mut allowance =
            PlanningResidentAllowance::new(retained, context.limits.recovery_memory_bytes)
                .map_err(|cause| ResolvedStateBackingDenial {
                    cause,
                    peak: state.capture_peak,
                })?;
        let bytes =
            PlanningResidentAllowance::slot_bytes::<ResolvedPlanningState>(1).map_err(|cause| {
                ResolvedStateBackingDenial {
                    cause,
                    peak: allowance.peak(),
                }
            })?;
        let combined = bytes
            .checked_mul(2)
            .ok_or_else(|| ResolvedStateBackingDenial {
                cause: overflow(),
                peak: allowance.peak(),
            })?;
        allowance
            .transient(combined)
            .map_err(|cause| ResolvedStateBackingDenial {
                cause,
                peak: allowance.peak(),
            })?;
        let mut values = allowance
            .reserve::<ResolvedPlanningState>(1)
            .map_err(|cause| ResolvedStateBackingDenial {
                cause,
                peak: allowance.peak(),
            })?;
        allowance
            .transient(bytes)
            .map_err(|cause| ResolvedStateBackingDenial {
                cause,
                peak: allowance.peak(),
            })?;
        state.unused_backing_bytes = PlanningResidentAllowance::vector_bytes(&values)
            .map_err(|cause| ResolvedStateBackingDenial {
                cause,
                peak: allowance.peak(),
            })?
            .checked_sub(bytes)
            .ok_or_else(|| ResolvedStateBackingDenial {
                cause: overflow(),
                peak: allowance.peak(),
            })?;
        state.capture_peak = state.capture_peak.max(allowance.peak());
        values.push(state);
        Ok(Self { values })
    }

    pub(in crate::orchestration::planning) fn state(&self) -> &ResolvedPlanningState {
        &self.values[0]
    }

    pub(in crate::orchestration::planning) fn state_mut(&mut self) -> &mut ResolvedPlanningState {
        &mut self.values[0]
    }

    pub(in crate::orchestration::planning) fn into_state(mut self) -> ResolvedPlanningState {
        let mut state = self
            .values
            .pop()
            .expect("resolved owner contains its admitted state");
        state.unused_backing_bytes = 0;
        state
    }
}

pub(super) fn deny(
    context: PlanningContext,
    counters: worth_store_recovery_physics::RecoveryPlanningCounters,
    denial: ResolvedStateBackingDenial,
) -> PhysicalRecoveryOutcome {
    match denial.cause {
        PlanningMemoryDenial::RecoveryMemoryBytes { observed } => {
            let limit = RecoveryAllowance::declared(
                &context.limits,
                PhysicalRecoveryLimitDimension::RecoveryMemoryBytes,
            )
            .past(observed);
            context.cost_denial_block(
                counters.with_peak_recovery_bytes(denial.peak.max(observed)),
                worth_store_recovery_physics::RecoveryPlanCostDenial::RecoveryMemoryBytes,
                limit,
            )
        }
        PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        } => context.block_with_planning_attempt_denial(
            PhysicalRecoveryBlockKind::RedoPlanning,
            counters.with_peak_recovery_bytes(denial.peak),
            "resolved-state-allocation",
            None,
            PhysicalRecoveryPlanningDenial::ResolvedStateAllocation {
                requested_bytes,
                cause,
            },
        ),
    }
}
