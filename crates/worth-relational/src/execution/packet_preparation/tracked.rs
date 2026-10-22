//! Native variable-shape artifacts claim live backing before growth and retain actual output.
use super::*;
use worth_execution::{ExecutionRequest, ExecutionWorkCeiling, ScopeStop};

pub(crate) fn prepare_tracked_artifact<T>(
    execution: &ExecutionRequest<'_, '_>,
    lease: &ExecutionResourceLease<'_>,
    work_budget: Option<&super::super::RequestWorkBudget>,
    build: impl FnOnce(
        &mut PacketPreparationBudget<'_, '_, '_, '_>,
    ) -> Result<(T, u64), MapKernelFailure<super::super::PacketBudgetDenial>>,
) -> Result<T, PacketExecutionStop> {
    let mut build = Some(build);
    let mut output = lease
        .reserve_memory(0)
        .map_err(|denial| PacketExecutionStop::Execution {
            boundary: None,
            reason: worth_execution::MapStop::Admission(
                worth_execution::LeaseDenial::MemoryExhausted(denial),
            ),
        })?;
    let outcome = super::super::run_with_remaining_request_work(
        lease,
        work_budget,
        |child| {
            // The child narrows cumulative work; the original caller request
            // remains the execution authority for its measured allocation.
            execution.run_measured_result(
                ExecutionWorkCeiling::new(child.policy().work_ceiling()),
                0,
                &mut output,
                |context, memory| {
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let mut budget = PacketPreparationBudget {
                        context,
                        ceiling: None,
                        claimed: 0,
                        memory: Some(memory),
                    };
                    let (value, owned_bytes) =
                        build.take().expect("one artifact preparation")(&mut budget)?;
                    Ok(PreparedArtifact { value, owned_bytes })
                },
            )
        },
        |outcome| outcome.report,
    );
    match outcome.result {
        Ok(artifact) => Ok(artifact.value),
        Err(ScopeStop::Admission(denial)) => Err(PacketExecutionStop::Execution {
            boundary: None,
            reason: worth_execution::MapStop::Admission(denial),
        }),
        Err(ScopeStop::Failure(cause)) => Err(PacketExecutionStop::Execution {
            boundary: None,
            reason: worth_execution::MapStop::Failure {
                identity: PartitionIdentity::new(1),
                cause,
            },
        }),
    }
}
