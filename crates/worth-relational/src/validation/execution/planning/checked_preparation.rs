use std::mem::size_of;

use worth_execution::{ExecutionResourceLease, MapKernelFailure, MapKernelStop};
use worth_foundational::{ExecutionReport, PartitionIdentity};

use crate::authority::commit::preparation::facade::PreparedInvariantExecution;
use crate::execution::{PacketBudgetDenial, PacketExecutionStop};
use crate::validation::custom_rule::{CustomPreparationBudget, CustomPreparationStop};
use crate::validation::engine::{InvariantExecutionRequest, InvariantRuntimeView};

use super::plan_invariant_execution_checked;

/// Planning charges the lease what it actually retains before each growth;
/// the lease's own memory policy is the bound, not a declared scratch share.
pub(crate) fn plan_checked_invariant_preparation<'state>(
    runtime: &InvariantRuntimeView<'state>,
    execution: worth_execution::ExecutionRequest<'state, 'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: &ExecutionResourceLease<'_>,
) -> Result<(PreparedInvariantExecution<'state>, u64, ExecutionReport), PacketExecutionStop> {
    tracked::prepare(runtime, execution, request, lease)
}

fn preparation_failure(stop: CustomPreparationStop) -> MapKernelFailure<PacketBudgetDenial> {
    match stop {
        CustomPreparationStop::Cancelled => MapKernelFailure::Stop(MapKernelStop::Cancelled),
        CustomPreparationStop::DeadlineElapsed => {
            MapKernelFailure::Stop(MapKernelStop::DeadlineElapsed)
        }
        CustomPreparationStop::WorkExhausted => MapKernelFailure::Stop(MapKernelStop::WorkCeiling),
        CustomPreparationStop::MemoryAdmission(denial) => MapKernelFailure::Domain(
            PacketBudgetDenial::ReadRequest(crate::execution::denial::lease_denial(
                worth_execution::LeaseDenial::MemoryExhausted(denial),
            )),
        ),
        CustomPreparationStop::MemoryExhausted => {
            MapKernelFailure::Domain(PacketBudgetDenial::ScratchCapacityExceeded)
        }
        CustomPreparationStop::UncheckedCustomKernel => {
            MapKernelFailure::Domain(PacketBudgetDenial::UncheckedCustomKernel)
        }
    }
}

mod tracked;
