//! Standalone Signal hosts declare their installed resource policy here.
use crate::runtime_policy::SignalRuntimePolicy;
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};

pub(crate) fn declared_serial_request(policy: SignalRuntimePolicy) -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(policy.serial_memory_bytes),
        CancellationToken::new(),
        None,
    )
}
