pub mod record_memory;
pub mod record_quote;

use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
use worth_execution::MapKernelContext;

/// Discard provisional values and denials on a cold miss before loading the
/// discovered footprint. All loads run under the original request.
pub fn with_full_residency<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::runtime::state::subsystems::storage::partition_edition::PartitionEdition,
    symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    read: impl FnMut(&dyn crate::storage::overlay::PartitionAccess) -> Result<T, Cause>,
) -> Result<T, Cause> {
    crate::storage::residency::probe_full(execution, &source.with_symbol_admission(symbols), read)
        .map_err(Cause::from)?
}

pub use crate::execution::request_stage::metered_with_memory;

pub fn metered<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(&mut MapKernelContext<'_, '_>) -> T,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<T, Cause> {
    metered_with_memory(execution, |context, _memory| read(context), owned_bytes)
}

pub fn point_with_quote<T>(
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

pub fn scan_with_memory<T, Limit>(
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

struct RetainedStageValue<T> {
    value: T,
    owned_bytes: u64,
}

impl<T> worth_execution::ChargedBytes for RetainedStageValue<T> {
    fn additional_charged_bytes(&self) -> u64 {
        self.owned_bytes
    }
}

/// Runs a metered scan and returns the settled `ExecutionMemoryReservation`
/// live alongside `T` so the caller never holds `T` unreserved across return.
pub fn scan_with_retained_memory<T, Limit>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    mut read: impl FnMut(
        &mut MapKernelContext<'_, '_>,
        &mut worth_execution::ExecutionResultMemory<'_>,
    ) -> Result<T, super::native_scan_stop::NativeScanStop<Limit>>,
    owned_bytes: impl Fn(&T) -> u64,
) -> Result<Result<(T, worth_execution::ExecutionMemoryReservation), Limit>, Cause> {
    crate::execution::request_custody::check_request(execution)?;
    let settled = execution
        .in_scope(|lease| {
            let mut memory =
                worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, 0)
                    .map_err(crate::execution::denial::lease_denial)?;
            let outcome = execution.run_measured_result::<_, ()>(
                worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling()),
                0,
                &mut memory,
                |context, result_memory| {
                    context
                        .checkpoint(0)
                        .map_err(worth_execution::MapKernelFailure::Stop)?;
                    let value = read(context, result_memory);
                    context
                        .checkpoint(0)
                        .map_err(worth_execution::MapKernelFailure::Stop)?;
                    let bytes = value.as_ref().map_or(0, &owned_bytes);
                    Ok(RetainedStageValue {
                        value,
                        owned_bytes: bytes,
                    })
                },
            );
            match outcome.result {
                Ok(answer) => Ok((answer.value, memory)),
                Err(worth_execution::ScopeStop::Admission(denial)) => {
                    Err(crate::execution::denial::lease_denial(denial))
                }
                Err(worth_execution::ScopeStop::Failure(cause)) => {
                    Err(crate::execution::denial::map_stop(
                        &worth_execution::MapStop::Failure {
                            identity: worth_foundational::PartitionIdentity::new(1),
                            cause,
                        },
                        std::convert::identity,
                        |()| unreachable!("the opaque stage emits no domain failure"),
                    ))
                }
            }
        })
        .map_err(Cause::from)??;
    match settled.0 {
        Ok(value) => Ok(Ok((value, settled.1))),
        Err(super::native_scan_stop::NativeScanStop::Request(cause)) => Err(cause),
        Err(super::native_scan_stop::NativeScanStop::Limit(limit)) => Ok(Err(limit)),
    }
}

/// A keyed record read uses the same one-unit checkpoint as its streamed
/// neighbor, before touching or cloning the record.
pub fn point<T>(
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
