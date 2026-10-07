//! Required installed memory for a caller's metered serial request.
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};

impl super::SignalGraph {
    pub(crate) fn bounded_serial_request(&self) -> SerialRequest {
        let policy = self.installed_runtime_policy().requested_policy();
        SerialRequest::from_memory(
            SerialMemoryBudget::new(policy.serial_memory_bytes),
            CancellationToken::new(),
            None,
        )
    }
}
