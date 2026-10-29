pub(crate) use {std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::CanonicalBits, worth_execution::ChargedBytes, worth_execution::DecomposeCertificationFailure, worth_execution::DecomposeKernelEditions, worth_execution::EquivalencePredicate, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose, worth_execution::ExecutionMap, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::OracleMismatch, worth_execution::ReduceCertificationFailure};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) const CONTRACT: EquivalenceContractId = worth_foundational::EquivalenceContractId::new(41);
pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());




pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct_with_equivalences(
            ExecutionAuthorityConfig {
                max_workers: std::num::NonZeroUsize::new(4).unwrap(),
                charged_memory_bytes: Some(1 << 20),
            },
            [worth_execution::EquivalencePredicate::new(
                CONTRACT,
                [41; 32],
                |left, right| {
                    let (Ok(left), Ok(right)): (Result<[u8; 8], _>, Result<[u8; 8], _>) =
                        (left.try_into(), right.try_into())
                    else {
                        return false;
                    };
                    (f64::from_le_bytes(left) - f64::from_le_bytes(right)).abs() < 0.01
                },
            )],
        )
        .unwrap()
    })
}

pub(crate) fn lease(
    work: u64,
    memory: u64,
    determinism: DeterminismContract,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                determinism,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(1).unwrap(), Some(memory), Some(work)),
            ),
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap()
}

pub(crate) fn map(count: u64) -> ExecutionMap<u64, u64> {
    let ids: Vec<_> = (1..=count).map(worth_foundational::PartitionIdentity::new).collect();
    let parts = ids
        .iter()
        .copied()
        .map(|identity| MapPartition {
            identity,
            value: identity.value(),
            read_keys: Vec::new(),
            write_keys: Vec::new(),
            kernel_scratch_bytes: 0,
            max_result_bytes: 8,
        })
        .collect();
    worth_execution::ExecutionMap::try_from_declared_partitions(ids, parts).unwrap()
}