pub(crate) use {std::num::NonZeroUsize, std::sync::Arc, std::sync::Mutex, std::sync::OnceLock};

pub(crate) use {
    worth_execution::Bisection, worth_execution::CancellationSource,
    worth_execution::CancellationToken, worth_execution::ComponentPartitioner,
    worth_execution::ExecutionAuthority, worth_execution::ExecutionAuthorityConfig,
    worth_execution::ExecutionMap, worth_execution::LeaseRequest,
    worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition,
    worth_execution::MapStop, worth_execution::PartitionItemId,
    worth_execution::PartitionUpdateDenial, worth_execution::WeightedEdge,
    worth_execution::WeightedItem,
};
pub(crate) use {
    worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget,
    worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy,
    worth_foundational::PartitionIdentity,
};

pub(crate) fn item(id: u64) -> PartitionItemId {
    PartitionItemId(id)
}
pub(crate) static CHECKED_TEST_LOCK: Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn checked_test_guard() -> std::sync::MutexGuard<'static, ()> {
    CHECKED_TEST_LOCK
        .lock()
        .unwrap_or_else(|error| error.into_inner())
}

pub(crate) fn authority() -> &'static ExecutionAuthority {
    static AUTHORITY: OnceLock<ExecutionAuthority> = std::sync::OnceLock::new();
    AUTHORITY.get_or_init(|| {
        worth_execution::ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: std::num::NonZeroUsize::new(4).unwrap(),
            charged_memory_bytes: Some(10_000_000),
        })
        .unwrap()
    })
}

pub(crate) fn map() -> ExecutionMap<u64, u64> {
    worth_execution::ExecutionMap::try_from_declared_partitions(
        vec![worth_foundational::PartitionIdentity::new(1)],
        vec![MapPartition {
            identity: worth_foundational::PartitionIdentity::new(1),
            value: 1,
            read_keys: vec![],
            write_keys: vec![],
            kernel_scratch_bytes: 512,
            max_result_bytes: 0,
        }],
    )
    .unwrap()
}

pub(crate) fn lease<'a>(
    authority: &'a ExecutionAuthority,
    memory: u64,
    ceiling: u64,
    cancellation: CancellationToken,
) -> worth_execution::ExecutionResourceLease<'a> {
    authority
        .request_lease(LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    Some(memory),
                    Some(ceiling),
                ),
            ),
            deadline: None,
            cancellation,
        })
        .unwrap()
}

pub(crate) fn component_chain(count: u64) -> ComponentPartitioner {
    let mut components = worth_execution::ComponentPartitioner::new();
    for id in 1..=count {
        components.upsert_item(item(id));
    }
    for id in 1..count {
        components.add_edge(item(id), item(id + 1)).unwrap();
    }
    components
}

pub(crate) fn weighted_root(count: u64, max_leaf_weight: u64) -> Bisection {
    let mut bisection = worth_execution::Bisection::new(max_leaf_weight, 2).unwrap();
    for id in 1..=count {
        bisection
            .upsert_item(WeightedItem {
                item: item(id),
                weight: 1,
            })
            .unwrap();
    }
    bisection
}
