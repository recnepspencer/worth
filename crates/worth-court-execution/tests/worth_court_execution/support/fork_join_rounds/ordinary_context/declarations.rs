pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering,
    std::sync::Mutex, std::sync::OnceLock, std::time::Duration, std::time::Instant,
};

pub(crate) use {
    worth_execution::CancellationSource, worth_execution::CancellationToken,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionForkJoin, worth_execution::ExecutionResourceLease,
    worth_execution::ExecutionRounds, worth_execution::ForkChild, worth_execution::ForkJoinOutcome,
    worth_execution::LeaseRequest, worth_execution::MapDenial, worth_execution::MapKernelFailure,
    worth_execution::MapKernelStop, worth_execution::MapStop, worth_execution::RoundsOutcome,
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
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(65_536),
        })
        .unwrap()
    })
}

pub(crate) fn lease(
    posture: ExecutionPosture,
    work: u64,
    cancellation: CancellationToken,
) -> ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                posture,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(4).unwrap(),
                    None,
                    Some(work),
                ),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn fork() -> ExecutionForkJoin<u64, u64> {
    let identities = [
        worth_foundational::PartitionIdentity::new(1),
        worth_foundational::PartitionIdentity::new(2),
    ];
    worth_execution::ExecutionForkJoin::try_from_children(
        identities.to_vec(),
        identities
            .into_iter()
            .map(|identity| ForkChild {
                identity,
                input: identity.value(),
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .unwrap()
}

pub(crate) fn recursive(
    lease: &ExecutionResourceLease<'_>,
    depth: u32,
) -> ForkJoinOutcome<u64, ()> {
    fork().run(Some(lease), |value, context| {
        context.checkpoint(1)?;
        if depth == 0 {
            return Ok::<_, MapKernelFailure<()>>(*value);
        }
        match recursive(lease, depth - 1) {
            worth_execution::ForkJoinOutcome::Complete { values, .. } => {
                Ok(*value + values.into_iter().sum::<u64>())
            }
            _ => Err(worth_execution::MapKernelFailure::Stop(
                worth_execution::MapKernelStop::NestedStopped,
            )),
        }
    })
}
