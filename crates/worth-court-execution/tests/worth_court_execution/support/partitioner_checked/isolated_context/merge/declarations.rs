pub(crate) use {
    crate::partitioner_checked::isolated_context::authority,
    crate::partitioner_checked::isolated_context::checked_test_guard,
    crate::partitioner_checked::isolated_context::item,
    crate::partitioner_checked::isolated_context::lease,
    crate::partitioner_checked::isolated_context::map,
    crate::partitioner_checked::isolated_context::weighted_root, std::num::NonZeroUsize,
    std::sync::Arc, std::sync::Mutex, worth_execution::CancellationSource,
    worth_execution::CancellationToken, worth_execution::ComponentPartitioner,
    worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome,
    worth_execution::MapStop, worth_execution::PartitionUpdateDenial,
    worth_execution::WeightedEdge, worth_foundational::DeterminismContract,
    worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture,
    worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity,
};

pub(crate) fn two_component_islands() -> ComponentPartitioner {
    let mut components = worth_execution::ComponentPartitioner::new();
    for id in [1, 2, 100] {
        components.upsert_item(item(id));
    }
    for id in 10..=21 {
        components.upsert_item(item(id));
    }
    components.add_edge(item(1), item(2)).unwrap();
    for id in 10..21 {
        components.add_edge(item(id), item(id + 1)).unwrap();
    }
    components
}
