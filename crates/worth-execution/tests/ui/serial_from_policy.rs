use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
fn main() {
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(std::num::NonZeroUsize::MIN, 4096, 100),
    );
    let memory = SerialMemoryBudget::from_policy(&policy);
    let cancellation = CancellationToken::new();
    let deadline = None;
    let _request = SerialRequest::from_memory(memory, cancellation, deadline);
}
