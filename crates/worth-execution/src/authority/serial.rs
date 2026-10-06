use std::time::Instant;

use super::{CancellationToken, SerialMemoryBudget};

/// What bounds a run that holds no lease: the request's cancellation and
/// deadline, and the memory its policy grants when it has one. Every
/// lease-free pattern inside [`crate::ExecutionWorkCeiling::run_serial`]
/// observes all three.
#[derive(Debug, Clone)]
pub struct SerialRequest {
    pub memory: Option<SerialMemoryBudget>,
    pub deadline: Option<Instant>,
    pub cancellation: CancellationToken,
}
