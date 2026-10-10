//! Host policy for Runtime World's serial fixture entries.
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(64 * 1024 * 1024),
        CancellationSource::new().token(),
        None,
    )
}
