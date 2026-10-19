//! Named serial host policy for direct primitive fixture controls.
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}
