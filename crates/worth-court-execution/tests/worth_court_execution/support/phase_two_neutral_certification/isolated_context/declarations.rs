pub(crate) use {std::num::NonZeroUsize, std::sync::Mutex, std::sync::OnceLock, std::time::Instant};

pub(crate) use {worth_execution::compare_canonical_values, worth_execution::BackInput, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::CanonicalBits, worth_execution::ChargedBytes, worth_execution::DecomposeFailure, worth_execution::DecomposeKernelEditions, worth_execution::EquivalencePredicate, worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig, worth_execution::ExecutionDecompose, worth_execution::ExecutionForkJoin, worth_execution::ExecutionMap, worth_execution::ExecutionRounds, worth_execution::ExecutionScan, worth_execution::ForkChild, worth_execution::InterfaceSolution, worth_execution::InteriorResult, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapKernelStop, worth_execution::MapOutcome, worth_execution::MapPartition, worth_execution::MapStop, worth_execution::RoundsOutcome, worth_execution::ScanOutcome};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::EquivalenceContractId, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};

pub(crate) const MEMORY: u64 = 1 << 20;
pub(crate) const EQUIVALENCE_ID: EquivalenceContractId = worth_foundational::EquivalenceContractId::new(91);
pub(crate) const EDITIONS: DecomposeKernelEditions = DecomposeKernelEditions {
    interior: 1,
    interface: 1,
    back: 1,
};
pub(crate) static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
pub(crate) static TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn authority() -> &'static ExecutionAuthority {
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct_with_equivalences(
            ExecutionAuthorityConfig {
                max_workers: std::num::NonZeroUsize::new(machine_width().saturating_add(1).max(4)).unwrap(),
                charged_memory_bytes: Some(MEMORY),
            },
            [worth_execution::EquivalencePredicate::new(
                EQUIVALENCE_ID,
                [91; 32],
                |left, right| {
                    let (Ok(left), Ok(right)): (Result<[u8; 8], _>, Result<[u8; 8], _>) =
                        (left.try_into(), right.try_into())
                    else {
                        return false;
                    };
                    let left = f64::from_le_bytes(left);
                    let right = f64::from_le_bytes(right);
                    left.is_finite() && right.is_finite() && (left - right).abs() <= 1e-12
                },
            )],
        )
        .unwrap()
    })
}

pub(crate) fn machine_width() -> usize {
    worth_execution::ExecutionAuthorityConfig::available_worker_count().get()
}

pub(crate) fn widths() -> Vec<usize> {
    let machine = machine_width();
    let mut widths = vec![1, 2, machine, machine.saturating_add(1)];
    widths.sort_unstable();
    widths.dedup();
    widths
}

pub(crate) fn execution_matrix() -> Vec<(ExecutionPosture, usize)> {
    let mut matrix = vec![(worth_foundational::ExecutionPosture::Serial, 1)];
    matrix.extend(
        widths()
            .into_iter()
            .map(|width| (worth_foundational::ExecutionPosture::Automatic, width)),
    );
    matrix
}

pub(crate) fn lease(
    posture: ExecutionPosture,
    width: usize,
    work: u64,
    cancellation: CancellationToken,
    deadline: Option<Instant>,
    determinism: DeterminismContract,
) -> worth_execution::ExecutionResourceLease<'static> {
    authority()
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                posture,
                determinism,
                worth_foundational::ExecutionBudget::new(std::num::NonZeroUsize::new(width).unwrap(), Some(MEMORY), Some(work)),
            ),
            deadline,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn integer_map(count: u64) -> ExecutionMap<u64, u64> {
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