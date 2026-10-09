//! Explicit bounded custody for standalone operation-control test callers.

/// The fixtures install the operational policy; each executing call draws only
/// that policy's serial memory budget. No pool or policy work ceiling is minted.
pub(crate) fn operational_serial_request() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            worth_signal::facade::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    )
}
