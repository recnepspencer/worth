pub(crate) use {std::sync::Arc, worth_execution::CancellationSource, worth_execution::CancellationToken, worth_execution::ComponentPartitioner, worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapStop, std::sync::Mutex, std::num::NonZeroUsize, worth_foundational::PartitionIdentity, worth_execution::PartitionUpdateDenial, worth_execution::WeightedEdge, crate::partitioner_checked::isolated_context::authority, crate::partitioner_checked::isolated_context::checked_test_guard, crate::partitioner_checked::isolated_context::item, crate::partitioner_checked::isolated_context::lease, crate::partitioner_checked::isolated_context::map, crate::partitioner_checked::isolated_context::weighted_root};

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