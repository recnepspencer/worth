pub(crate) use {
    std::num::NonZeroUsize, std::sync::atomic::AtomicU64, std::sync::atomic::AtomicUsize,
    std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock,
};

pub(crate) use {
    worth_execution::CancellationToken, worth_execution::ExecutionAuthority,
    worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionMap,
    worth_execution::LeaseDenial, worth_execution::LeaseRequest, worth_execution::MapKernelFailure,
    worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapPartition,
    worth_execution::MapStop,
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
            charged_memory_bytes: Some(8_000),
        })
        .expect("one authority for this integration target")
    })
}

pub(crate) fn request(work_ceiling: u64) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Automatic,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::new(2).unwrap(),
                Some(4_000),
                Some(work_ceiling),
            ),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

pub(crate) fn map(value: u64) -> ExecutionMap<u64, u64> {
    let identity = worth_foundational::PartitionIdentity::new(1);
    worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![identity],
        vec![MapPartition {
            identity,
            value,
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 0,
        }],
    )
    .expect("valid one-partition map")
}

pub(crate) fn two_partition_map() -> ExecutionMap<u64, u64> {
    let identities = [
        worth_foundational::PartitionIdentity::new(1),
        worth_foundational::PartitionIdentity::new(2),
    ];
    worth_execution::ExecutionMap::try_from_declared_partitions(
        identities.to_vec(),
        identities
            .into_iter()
            .enumerate()
            .map(|(index, identity)| MapPartition {
                identity,
                value: index as u64 + 1,
                read_keys: Vec::new(),
                write_keys: Vec::new(),
                kernel_scratch_bytes: 0,
                max_result_bytes: 0,
            })
            .collect(),
    )
    .expect("valid two-partition map")
}
