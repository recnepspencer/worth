pub(crate) mod record_memory;
pub(crate) mod record_quote;

use crate::execution::RelationalExecutionDenialCause as Cause;
use worth_execution::MapKernelContext;

/// Discard provisional values and denials on a cold miss before loading the
/// discovered footprint. All loads run under the original request.
pub(crate) fn with_full_residency<T>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: &crate::runtime::PartitionEdition,
    symbols: &crate::runtime::RuntimeSymbolTable,
    read: impl FnMut(&dyn crate::storage::overlay::PartitionAccess) -> Result<T, Cause>,
) -> Result<T, Cause> {
    crate::storage::residency::probe_full(execution, &source.with_symbol_admission(symbols), read)
        .map_err(Cause::from)?
}

pub(crate) use crate::execution::request_stage::metered_with_memory;

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
