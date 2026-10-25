//! Retain actual stage backing on the original request, across later preparation.
use super::{finish::CompletedInvariantReduction, *};
use crate::execution::{
    PacketBudgetDenial, PacketExecutionStop, RelationalExecutionDenialCause as Cause,
    RequestWorkBudget,
};
use crate::transactions::data::TransactionCommitError;
use crate::validation::engine::{check_invariant_request, InvariantExecutionRequest};
use worth_execution::{
    ChargedBytes, ExecutionRequest, ExecutionResourceLease, ExecutionWorkCeiling, LeaseDenial,
    MapKernelFailure, MapStop, ScopeStop,
};
use worth_foundational::PartitionIdentity;

struct AbsorbedStage {
    prefix: ReducedInvariantPrefix,
    owned_bytes: u64,
}

impl ChargedBytes for AbsorbedStage {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

impl ReducedInvariantPrefix {
    pub(crate) fn absorb_in_request(
        self,
        output: InvariantStageOutput,
        execution: ExecutionRequest<'_, '_>,
        lease: Option<&ExecutionResourceLease<'_>>,
        budget: Option<&RequestWorkBudget>,
    ) -> Result<Self, TransactionCommitError> {
        let input_memory = reservation(execution, lease, output.owned_allocation_capacity_bytes())?;
        let input = StageInput {
            output,
            memory: input_memory,
        };
        if input.memory.is_none() {
            return self
                .absorb(input, |_, _| check_invariant_request(execution))
                .map_err(|stop| match stop {
                    ReductionStop::Check(error) => error,
                    ReductionStop::Order => order_denial().into(),
                });
        }
        let mut auxiliary = reservation(execution, lease, 0)?
            .expect("the same request has an active memory lineage");
        let mut build = Some((self, input));
        let absorbed = run(
            execution,
            lease,
            budget,
            &mut auxiliary,
            |context, memory| {
                let (prefix, input) = build.take().expect("one selected stage reduction");
                let mut claimed = 0_u64;
                let prefix = prefix
                    .absorb(input, |work, bytes| {
                        claim(context, memory, &mut claimed, work, bytes)
                    })
                    .map_err(|stop| match stop {
                        ReductionStop::Check(error) => error,
                        ReductionStop::Order => MapKernelFailure::Domain(
                            PacketBudgetDenial::ReadRequest(Cause::ExpectedIdentitiesNotCanonical),
                        ),
                    })?;
                let owned_bytes = prefix
                    .completed()
                    .next_back()
                    .expect("the completed stage is retained")
                    .auxiliary_bytes();
                Ok(AbsorbedStage {
                    prefix,
                    owned_bytes,
                })
            },
        )?;
        let mut prefix = absorbed.prefix;
        prefix.stages[prefix.stage_count - 1]
            .as_mut()
            .expect("completed selected stage")
            .auxiliary_memory = Some(auxiliary);
        Ok(prefix)
    }

    pub(crate) fn finish_in_request(
        self,
        request: &InvariantExecutionRequest<'_>,
        execution: ExecutionRequest<'_, '_>,
        lease: Option<&ExecutionResourceLease<'_>>,
        budget: Option<&RequestWorkBudget>,
    ) -> Result<CompletedInvariantReduction, TransactionCommitError> {
        let Some(mut memory) = reservation(execution, lease, 0)? else {
            return self.project(request, |_, _| check_invariant_request(execution));
        };
        let mut input = Some(self);
        let mut completed = run(execution, lease, budget, &mut memory, |context, backing| {
            let mut claimed = 0_u64;
            input
                .take()
                .expect("one final result projection")
                .project(request, |work, bytes| {
                    claim(context, backing, &mut claimed, work, bytes)
                })
        })?;
        completed.projection_memory = Some(memory);
        Ok(completed)
    }
}

fn reservation(
    execution: ExecutionRequest<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    bytes: u64,
) -> Result<Option<ExecutionMemoryReservation>, PacketExecutionStop> {
    match ExecutionMemoryReservation::reserve_in_scope(lease, bytes) {
        Ok(memory) => Ok(Some(memory)),
        // Native serial omission is deliberate; an accounted ancestor, if any,
        // already had the chance to issue the reservation above.
        Err(LeaseDenial::NoActiveExecutionScope)
            if lease.is_none() && !execution.has_accounting() =>
        {
            Ok(None)
        }
        Err(denial) => Err(admission(denial)),
    }
}

fn claim(
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    memory: &mut worth_execution::ExecutionResultMemory<'_>,
    claimed: &mut u64,
    work: u64,
    bytes: u64,
) -> Result<(), MapKernelFailure<PacketBudgetDenial>> {
    context.checkpoint(work).map_err(MapKernelFailure::Stop)?;
    *claimed = claimed.checked_add(bytes).ok_or(MapKernelFailure::Domain(
        PacketBudgetDenial::ReadRequest(Cause::ChargedBytesOverflow),
    ))?;
    memory.resize(*claimed).map_err(|denial| {
        MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
            crate::execution::denial::lease_denial(LeaseDenial::MemoryExhausted(denial)),
        ))
    })
}

fn run<R: ChargedBytes>(
    execution: ExecutionRequest<'_, '_>,
    lease: Option<&ExecutionResourceLease<'_>>,
    budget: Option<&RequestWorkBudget>,
    memory: &mut ExecutionMemoryReservation,
    kernel: impl FnOnce(
        &mut worth_execution::MapKernelContext<'_, '_>,
        &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<R, MapKernelFailure<PacketBudgetDenial>>,
) -> Result<R, PacketExecutionStop> {
    let mut kernel = Some(kernel);
    let mut run = |ceiling| {
        execution.run_measured_result(
            ExecutionWorkCeiling::new(ceiling),
            0,
            memory,
            |context, backing| {
                context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                kernel.take().expect("one measured reduction stage")(context, backing)
            },
        )
    };
    let outcome = if let Some(lease) = lease {
        crate::execution::run_with_remaining_request_work(
            lease,
            budget,
            |child| run(child.policy().work_ceiling()),
            |outcome| outcome.report,
        )
    } else {
        run(execution.work_ceiling())
    };
    match outcome.result {
        Ok(value) => Ok(value),
        Err(ScopeStop::Admission(denial)) => Err(admission(denial)),
        Err(ScopeStop::Failure(cause)) => Err(PacketExecutionStop::Execution {
            boundary: Some(PartitionIdentity::new(1)),
            reason: MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause,
            },
        }),
    }
}

fn admission(denial: LeaseDenial) -> PacketExecutionStop {
    PacketExecutionStop::Execution {
        boundary: None,
        reason: MapStop::Admission(denial),
    }
}
