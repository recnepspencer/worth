//! Host policy for Relational's serial fixture entries.
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

pub(crate) fn serial_request_with_memory(memory: SerialMemoryBudget) -> SerialRequest {
    SerialRequest::from_memory(memory, CancellationSource::new().token(), None)
}
