//! The named bounded memory policy for standalone workflow proofs.
use worth_execution::{CancellationToken, SerialMemoryBudget, SerialRequest};
const WORKFLOW_PROOF_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
/// A fresh bounded serial carrier for standalone workflow proofs.
/// Production callers supply their own request and policy.
pub fn workflow_proof_execution_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(WORKFLOW_PROOF_MEMORY_BYTES),
        CancellationToken::new(),
        None,
    )
}
