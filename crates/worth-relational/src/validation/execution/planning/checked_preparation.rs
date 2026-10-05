use std::mem::size_of;

use worth_execution::{
    ChargedBytes, ExecutionResourceLease, ExecutionScan, MapKernelFailure, MapKernelStop,
    ScanDenial, ScanOutcome,
};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::execution::{PacketBudgetDenial, PacketExecutionStop};
use crate::validation::custom_rule::{CustomPreparationBudget, CustomPreparationStop};
use crate::validation::engine::{InvariantExecutionRequest, InvariantRuntimeView};

use super::plan_invariant_execution_checked;

struct CheckedPlan<'state> {
    plan: PreparedInvariantExecution<'state>,
    owned_bytes: u64,
}

impl ChargedBytes for CheckedPlan<'_> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

pub(crate) fn plan_checked_invariant_preparation<'state>(
    runtime: &InvariantRuntimeView<'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<(PreparedInvariantExecution<'state>, u64, ExecutionReport), PacketExecutionStop> {
    let identity = PartitionIdentity::new(1);
    let scan = ExecutionScan::try_from_ordered(vec![identity], vec![(identity, ())]).map_err(
        |denial| {
            PacketExecutionStop::Admission(match denial {
                ScanDenial::IdentitiesNotCanonical => {
                    worth_execution::MapDenial::ExpectedIdentitiesNotCanonical
                }
                ScanDenial::CoverageMismatch => worth_execution::MapDenial::CoverageMismatch,
                ScanDenial::MemoryOverflow => worth_execution::MapDenial::MemoryOverflow,
            })
        },
    )?;
    let ceiling = lease.policy().budget().charged_memory_bytes() / 8;
    let outcome = crate::execution::run_with_remaining_request_work(
        lease,
        runtime.commit_work_budget.as_ref(),
        |child_lease| {
            scan.run(
                Some(child_lease), (), 0, ceiling, 0, ceiling,
                |_, _, context| {
                    context.checkpoint(0)?;
                    let meter = CustomPreparationBudget::new(
                        child_lease.status(),
                        child_lease.policy().budget().work_ceiling(),
                        ceiling,
                    );
                    let plan = plan_invariant_execution_checked(runtime, request, child_lease, &meter);
                    let work = meter.charged_work();
                    context.account_completed_work(work).map_err(MapKernelFailure::Stop)?;
                    if let Some(stop) = meter.stop() {
                        return Err(preparation_failure(stop));
                    }
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let owned_bytes = (plan.packets.capacity() as u64)
                        .saturating_mul(size_of::<crate::authority::commit::preparation::packets::invariant::InvariantWorkPacket<'_>>() as u64)
                        .saturating_add(meter.claimed_memory());
                    if owned_bytes > ceiling {
                        return Err(MapKernelFailure::ResultCapacityExceeded);
                    }
                    Ok(((), CheckedPlan { plan, owned_bytes }))
                },
            )
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
            Ok((completed.plan, completed.owned_bytes, report))
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
