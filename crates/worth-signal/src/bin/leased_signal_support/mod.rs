use std::num::NonZeroUsize;
use worth_execution::{
    CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig, ExecutionResourceLease,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_signal::facade::adapters::NodeContract;
use worth_signal::facade::{Aspect, BoundedSignalInputs, DeclaredSignalInput, NodeId};

/// Each command constructs its host once and carries it into every workload.
pub(super) fn host() -> ExecutionAuthority {
    ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
        max_workers: NonZeroUsize::new(4).unwrap(),
        charged_memory_bytes: 256 * 1024 * 1024,
    })
    .expect("command execution host")
}

pub(super) fn lease(host: &ExecutionAuthority, workers: usize) -> ExecutionResourceLease<'_> {
    host.request_lease(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(
                NonZeroUsize::new(workers).unwrap(),
                128 * 1024 * 1024,
                100_000_000,
            ),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    })
    .expect("command host lease")
}

pub(super) fn contract(inputs: &[NodeId], aspect: Aspect) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(
        inputs
            .iter()
            .map(|&source| DeclaredSignalInput::new(source, aspect)),
    ))
}
