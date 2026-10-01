use worth_store_recovery_physics::{
    admit_recovery_plan_cost, RecoveryPlanCost, RecoveryPlanLimits,
};

use super::super::context::PlanningContext;
use super::super::denial::plan_cost_limit;
use super::super::resolved_basis::ResolvedPlanningBasis;
use super::execution_basis::ExecutionProducts;
use crate::entry::PhysicalRecoveryOutcome;

pub(super) fn admit(
    context: PlanningContext,
    basis: &ResolvedPlanningBasis,
    execution: &ExecutionProducts,
) -> Result<
    (
        PlanningContext,
        RecoveryPlanCost,
        worth_store_recovery_physics::RecoveryPlanningCounters,
    ),
    PhysicalRecoveryOutcome,
> {
    let plan_limits = RecoveryPlanLimits::new(
        context.limits.redo_targets,
        context.limits.redo_bytes,
        context.limits.distinct_pages_and_extents,
        context.limits.operation_bindings,
        context.limits.observation_bytes,
        context.limits.staging_bytes,
        context.limits.recovery_memory_bytes,
        context.limits.dirty_frames,
    )
    .expect("admitted runtime limits are nonzero");
    let candidate_comparison_peak = basis
        .observed_pages
        .candidate_peak_materialization_bytes
        .checked_add(
            execution
                .candidate_materialization
                .comparison_scratch_bytes(),
        )
        .expect("candidate comparison memory accounting cannot overflow");
    let candidate_lifecycle_peak =
        candidate_comparison_peak.max(execution.candidate_materialization.publication_bytes());
    let staging = &execution.staging;
    let legacy_cost_peak = basis
        .observed_pages
        .bytes_read
        .checked_add(basis.observed_pages.source_copy_peak_scratch_bytes)
        .and_then(|bytes| {
            bytes.checked_add(
                basis
                    .observed_pages
                    .historical_publication_peak_scratch_bytes,
            )
        })
        .and_then(|bytes| {
            bytes.checked_add(
                (basis.verified_drops.capacity()
                    * std::mem::size_of::<worth_store_physical_format::PersistedRecordIdentity>())
                    as u64,
            )
        })
        .and_then(|bytes| bytes.checked_add(candidate_lifecycle_peak))
        .and_then(|bytes| bytes.checked_add(basis.redo.supersession_scratch_bytes()))
        .and_then(|bytes| {
            bytes.checked_add(
                basis
                    .historical_consumed
                    .as_ref()
                    .expect("post-verification historical disposition is present")
                    .memory_bytes(),
            )
        })
        .and_then(|bytes| bytes.checked_add(basis.sample.manifest_cleanup_sampling_peak_bytes()))
        .and_then(|bytes| bytes.checked_add(staging.allocated_bytes()))
        .and_then(|bytes| bytes.checked_add(staging.write_bytes()))
        .unwrap_or(u64::MAX);
    // The existing lifecycle estimate and actual final live storage are two
    // independent conservative checks. This floor does not claim to reserve
    // subsequent Store rejoin scratch or every earlier planning allocation.
    let final_live_bytes = super::super::resident_memory::live_bytes(&context, basis)
        .and_then(|bytes| {
            bytes.checked_add(u64::try_from(std::mem::size_of::<ExecutionProducts>()).ok()?)
        })
        .and_then(|bytes| bytes.checked_add(execution.staging.owned_heap_bytes()?))
        .and_then(|bytes| bytes.checked_add(execution.publication.owned_heap_bytes()?))
        .unwrap_or(u64::MAX);
    let peak_recovery_bytes = legacy_cost_peak
        .max(final_live_bytes)
        .max(execution.planning_construction_peak);
    let planning_counters = basis
        .planning_counters()
        .with_peak_recovery_bytes(peak_recovery_bytes);
    let plan_cost = RecoveryPlanCost::new(
        basis.targets.len() as u64,
        basis.redo_bytes,
        basis.distinct_targets,
        basis.sample.operations().len() as u64,
        basis
            .observed_pages
            .artifact_reads
            .saturating_add(basis.observed_pages.candidate_artifact_reads)
            .saturating_add(basis.observed_pages.source_copy_reads)
            .saturating_add(basis.observed_pages.historical_publication_reads),
        context
            .counters
            .bytes_observed
            .saturating_add(basis.observed_pages.bytes_read)
            .saturating_add(basis.observed_pages.candidate_bytes_read)
            .saturating_add(basis.observed_pages.source_copy_bytes_read)
            .saturating_add(basis.observed_pages.historical_publication_bytes_read),
        staging.allocated_bytes(),
        peak_recovery_bytes,
        staging.dirty_frames(),
    );
    let plan_cost = match admit_recovery_plan_cost(plan_limits, plan_cost) {
        Ok(cost) => cost,
        Err(denial) => {
            let limit = plan_cost_limit(denial, plan_limits, plan_cost);
            return Err(context.cost_denial_block(planning_counters, denial, limit));
        }
    };
    Ok((context, plan_cost, planning_counters))
}
