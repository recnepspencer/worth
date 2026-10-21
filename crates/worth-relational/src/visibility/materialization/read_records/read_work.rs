pub(crate) mod record_memory;
pub(crate) mod record_quote;

use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::{ChargedBytes, MapKernelContext, MapKernelFailure};
use worth_foundational::PartitionIdentity;

struct MaterializedRead<T> {
    value: T,
    owned_bytes: u64,
}

impl<T> ChargedBytes for MaterializedRead<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Run the native scan's checkpoints in the caller's active request. Native
/// limits stay in the value; request stops return before any value can escape.
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
                    Ok(MaterializedRead { value, owned_bytes })
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
                        |()| unreachable!("the opaque read stage emits no domain failure"),
                    ))
                }
            }
        })
        .map_err(super::read_request::cause)?
}

pub(crate) fn metered<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(&mut MapKernelContext<'_, '_>) -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    metered_with_memory(execution, |context, _memory| read(context), owned_bytes)
}

pub(crate) fn point_with_quote<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut quote: impl FnMut() -> Result<u64, Cause>,
    mut read: impl FnMut() -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    metered_with_memory(
        execution,
        |context, memory| {
            context
                .checkpoint(super::read_prices::KEYED_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            let quoted = quote()?;
            record_memory::claim(memory, quoted)?;
            let value = read();
            record_memory::settle_clone(memory, quoted, owned_bytes(&value))?;
            Ok(value)
        },
        |answer: &Result<T, Cause>| answer.as_ref().map_or(0, &owned_bytes),
    )?
}

pub(super) fn scan_with_memory<T, Limit>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<T, super::native_scan_stop::NativeScanStop<Limit>>,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<Result<T, Limit>, Cause> {
    match metered_with_memory(
        execution,
        |context, memory| read(context, memory),
        |answer| answer.as_ref().map_or(0, &owned_bytes),
    )? {
        Ok(value) => Ok(Ok(value)),
        Err(super::native_scan_stop::NativeScanStop::Request(cause)) => Err(cause),
        Err(super::native_scan_stop::NativeScanStop::Limit(limit)) => Ok(Err(limit)),
    }
}

/// A keyed record read uses the same one-unit checkpoint as its streamed
/// neighbor, before touching or cloning the record.
pub(crate) fn point<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut() -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    metered(
        execution,
        |context| {
            context
                .checkpoint(super::read_prices::KEYED_RECORD)
                .map_err(crate::execution::denial::kernel_stop)?;
            Ok(read())
        },
        |value: &Result<T, Cause>| value.as_ref().map_or(0, &owned_bytes),
    )?
}
