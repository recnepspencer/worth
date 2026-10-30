pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock,
};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionScan, worth_execution::LeaseDenial, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapStop,
    worth_execution::ScanOutcome,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(2).unwrap(),
            charged_memory_bytes: Some(4096),
        })
        .unwrap()
    })
}

pub(crate) fn lease(
    work: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Automatic,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(2).unwrap(),
                    Some(4096),
                    Some(work),
                ),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn scan() -> ExecutionScan<u64> {
    let ids = [1, 2, 3].map(worth_foundational::PartitionIdentity::new);
    worth_execution::ExecutionScan::try_from_ordered(
        ids.to_vec(),
        ids.into_iter().zip([1, 2, 3]).collect(),
    )
    .unwrap()
}
