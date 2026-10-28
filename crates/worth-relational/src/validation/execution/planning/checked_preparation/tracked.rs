//! Checked custom planning claims its variable retained shape before each growth.
use super::*;
use worth_execution::{ExecutionWorkCeiling, MapStop};

pub(super) fn prepare<'state>(
    runtime: &InvariantRuntimeView<'state>,
    execution: worth_execution::ExecutionRequest<'state, 'state>,
    request: &'state InvariantExecutionRequest<'state>,
    lease: &ExecutionResourceLease<'_>,
    stage: super::super::InvariantStage,
) -> Result<(PreparedInvariantExecution<'state>, u64, ExecutionReport), PacketExecutionStop> {
    let empty = ExecutionReport::new(
        lease.resolved_posture(),
        0,
        0,
        worth_foundational::ExecutionPhysicalReport::new(0, 0, None, 0, 0),
    );
    let mut retained = None;
    let outcome = crate::execution::run_with_remaining_request_work(
        lease,
        runtime.commit_work_budget.as_ref(),
        |child| {
            ExecutionWorkCeiling::new(child.policy().work_ceiling()).run_with_checkpoint(
                Some(child),
                None,
                |context| {
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let meter = CustomPreparationBudget::tracked(child).map_err(|denial| {
                        MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
                            crate::execution::denial::lease_denial(
                                worth_execution::LeaseDenial::MemoryExhausted(denial),
                            ),
                        ))
                    })?;
                    let plan = plan_invariant_execution_checked(
                        runtime, execution, request, child, &meter, stage,
                    );
                    context
                        .account_completed_work(meter.charged_work())
                        .map_err(MapKernelFailure::Stop)?;
                    if let Some(stop) = meter.stop() {
                        return Err(preparation_failure(stop));
                    }
                    let owned = (plan.packets.capacity() as u64)
                        .checked_mul(size_of::<
                            crate::authority::commit::preparation::InvariantWorkPacket<'_>,
                        >() as u64)
                        .and_then(|bytes| bytes.checked_add(meter.claimed_memory()))
                        .ok_or(MapKernelFailure::Domain(PacketBudgetDenial::ReadRequest(
                            crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
                        )))?;
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    retained = Some(meter);
                    Ok((plan, owned))
                },
            )
        },
        |outcome| outcome.as_ref().map_or(empty, |(_, report)| *report),
    );
    match outcome {
        Ok((Ok((plan, bytes)), report)) => Ok((plan, bytes, report)),
        // The single planning stage is partition 1.
        Ok((Err(cause), _)) => Err(PacketExecutionStop::Execution {
            boundary: Some(PartitionIdentity::new(1)),
            reason: MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause,
            },
        }),
        Err(worth_execution::WorkCeilingDenial::Admission(denial)) => {
            Err(PacketExecutionStop::Execution {
                boundary: None,
                reason: MapStop::Admission(denial),
            })
        }
        Err(worth_execution::WorkCeilingDenial::Stopped(stop)) => {
            Err(PacketExecutionStop::Execution {
                boundary: Some(PartitionIdentity::new(1)),
                reason: MapStop::Failure {
                    identity: PartitionIdentity::new(1),
                    cause: MapKernelFailure::Stop(stop),
                },
            })
        }
        Err(worth_execution::WorkCeilingDenial::Panicked) => Err(PacketExecutionStop::Execution {
            boundary: Some(PartitionIdentity::new(1)),
            reason: MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause: MapKernelFailure::Panic,
            },
        }),
    }
}
