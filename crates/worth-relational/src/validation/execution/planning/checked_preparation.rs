use worth_execution::{
    ChargedBytes, ExecutionResourceLease, MapKernelFailure, MapKernelStop, ScanOutcome,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::execution::{InvariantScratchAdmission, PacketBudgetDenial, PacketExecutionStop};
use crate::validation::custom_rule::{CustomPreparationBudget, CustomPreparationStop};
use crate::validation::engine::{InvariantExecutionRequest, InvariantRuntimeView};

use super::plan_invariant_execution_checked;

struct CheckedPlan<'state> {
    plan: PreparedInvariantExecution<'state>,
    custody: InvariantScratchAdmission,
}

impl ChargedBytes for CheckedPlan<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

pub(crate) fn plan_checked_invariant_preparation<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<
    (
        PreparedInvariantExecution<'state>,
        InvariantScratchAdmission,
        ExecutionReport,
    ),
    PacketExecutionStop,
> {
    let identity = PartitionIdentity::new(1);
    let scan = crate::execution::admit_ordered_scan(vec![(identity, ())])?;
    let outcome = crate::execution::run_with_remaining_request_work(
        lease,
        runtime.commit_work_budget.as_ref(),
        |child_lease| {
            scan.run(Some(child_lease), (), 0, 0, 0, 0, |_, _, context| {
                context.checkpoint(0)?;
                let custody = InvariantScratchAdmission::new(child_lease)
                    .map_err(MapKernelFailure::Domain)?;
                custody
                    .claim(CustomPreparationBudget::allocation_bytes())
                    .map_err(MapKernelFailure::Domain)?;
                let meter = CustomPreparationBudget::with_scratch_admission(
                    child_lease.status(),
                    Some(child_lease.policy().budget().work_ceiling()),
                    custody.clone(),
                );
                let plan = plan_invariant_execution_checked(runtime, request, child_lease, &meter);
                let work = meter.charged_work();
                context
                    .account_completed_work(work)
                    .map_err(MapKernelFailure::Stop)?;
                if let Some(failure) = runtime
                    .shared_candidate_inputs()
                    .and_then(|inputs| inputs.failure())
                {
                    return Err(MapKernelFailure::Domain(failure));
                }
                if let Some(failure) = custody.failure() {
                    return Err(MapKernelFailure::Domain(failure));
                }
                if let Some(stop) = meter.stop() {
                    return Err(preparation_failure(stop));
                }
                context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                let plan = plan.expect("planning refusal records its typed cause");
                Ok(((), CheckedPlan { plan, custody }))
            })
        },
        ScanOutcome::report,
    );
    match outcome {
        ScanOutcome::Complete {
            mut prefixes,
            report,
            ..
        } => {
            let completed = prefixes.pop().expect("one checked plan");
            Ok((completed.plan, completed.custody, report))
        }
        ScanOutcome::Stopped {
            boundary, reason, ..
        } => Err(PacketExecutionStop::Execution { boundary, reason }),
    }
}

fn preparation_failure(stop: CustomPreparationStop) -> MapKernelFailure<PacketBudgetDenial> {
    match stop {
        CustomPreparationStop::Cancelled => MapKernelFailure::Stop(MapKernelStop::Cancelled),
        CustomPreparationStop::DeadlineElapsed => {
            MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed)
        }
        CustomPreparationStop::WorkExhausted => MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
        CustomPreparationStop::MemoryExhausted => {
            MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded)
        }
        CustomPreparationStop::UncheckedCustomKernel => {
            MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel)
        }
    }
}

#[cfg(test)]
mod memory_tests;
