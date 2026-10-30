//! Host policy for Query's serial fixture entries.
pub use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}
