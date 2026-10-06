use std::{
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
};
use worth_execution::{
    CancellationSource, CancellationToken, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};

pub(crate) fn authority() -> &'static ExecutionAuthority {
    shared_authority().as_ref()
}

pub(crate) fn shared_authority() -> &'static Arc<ExecutionAuthority> {
    static AUTHORITY: OnceLock<Arc<ExecutionAuthority>> = OnceLock::new();
    AUTHORITY.get_or_init(|| {
        Arc::new(
            ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
                max_workers: NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: 512 * 1024 * 1024,
            })
            .expect("one Signal test process authority"),
        )
    })
}

pub(crate) fn request(workers: usize, work: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Automatic,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(workers).unwrap(), 32 * 1024 * 1024, work),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

/// A request whose cancellation the returned source owns.
pub(crate) fn cancellable(workers: usize, work: u64) -> (LeaseRequest, CancellationSource) {
    let source = CancellationSource::new();
    let request = LeaseRequest {
        cancellation: source.token(),
        ..request(workers, work)
    };
    (request, source)
}
