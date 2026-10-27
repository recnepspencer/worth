//! One original-request stage shared by record reads and explicit maintenance.
use super::result_memory::OriginalResultMemory;
use super::RelationalExecutionDenialCause as Cause;
use worth_execution::{ChargedBytes, MapKernelContext, MapKernelFailure};
use worth_foundational::PartitionIdentity;

/// Inspect the effective original scope, including authentic accounted ancestry.
/// A serial backing alone cannot decide whether an inherited loan is required.
pub(crate) fn optional_stage<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut stage: impl FnMut(&mut MapKernelContext<'_, '_>, &mut OriginalResultMemory<'_, '_>) -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    let mut answer = None;
    let handoff = execution.run_native_with_checkpoint(|context| {
        answer = Some(if context.has_accounting() {
            metered_with_memory(
                execution,
                |context, memory| stage(context, &mut OriginalResultMemory::new(Some(memory))),
                &owned_bytes,
            )
        } else {
            Ok(stage(context, &mut OriginalResultMemory::new(None)))
        });
    });
    match handoff {
        Ok(_) => answer.expect("a completed original stage executed its callback"),
        Err(stop) => {
            let cause = Cause::from(stop);
            // Preserve a concrete inner refusal hidden by the SDK's generic
            // nested-stop handoff. The original latch remains stopped.
            match answer {
                Some(Err(inner)) if cause == Cause::NestedStopped => Err(inner),
                _ => Err(cause),
            }
        }
    }
}

/// Native-only work still observes the original cancellation and deadline.
pub(crate) fn checkpoint(
    context: &mut MapKernelContext<'_, '_>,
    work: u64,
) -> Result<(), worth_execution::MapKernelStop> {
    if context.has_accounting() {
        context.checkpoint(work)
    } else {
        context.poll()
    }
}

struct MeasuredStageValue<T> {
    value: T,
    owned_bytes: u64,
}

impl<T> ChargedBytes for MeasuredStageValue<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Run an opaque stage under the original request and its result-memory loan.
/// Domain stops stay in the value; original request stops remain authoritative.
pub(crate) fn metered_with_memory<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    crate::execution::check_request(execution)?;
    execution
        .in_scope(|lease| {
            let mut memory =
                worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                    .map_err(crate::execution::denial::lease_denial)?;
            let outcome = execution.run_measured_result::<_, ()>(
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling()),
                0,
                &mut memory,
                |context, memory| {
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let value = read(context, memory);
                    context.checkpoint(0).map_err(MapKernelFailure::Stop)?;
                    let owned_bytes = owned_bytes(&value);
                    Ok(MeasuredStageValue { value, owned_bytes })
                },
            );
            match outcome.result {
                Ok(answer) => Ok(answer.value),
                Err(worth_execution::ScopeStop::Admission(denial)) => {
                    Err(crate::execution::denial::lease_denial(denial))
                }
                Err(worth_execution::ScopeStop::Failure(cause)) => {
                    Err(crate::execution::denial::map_stop(
                        &worth_execution::MapStop::Failure {
                            identity: PartitionIdentity::new(1),
                            cause,
                        },
                        std::convert::identity,
                        |()| unreachable!("the opaque stage emits no domain failure"),
                    ))
                }
            }
        })
        .map_err(Cause::from)?
}
