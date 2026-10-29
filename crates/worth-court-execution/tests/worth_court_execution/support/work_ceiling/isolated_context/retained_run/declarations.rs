pub(crate) use {std::collections::BTreeMap, std::num::NonZeroUsize, std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};
pub(crate) use {worth_execution::CancellationToken, worth_execution::ExecutionMap, worth_execution::ExecutionRequest, worth_execution::ExecutionWorkCeiling, worth_execution::KeylessPartition, worth_execution::LeaseRequest, worth_execution::MapKernelFailure, worth_execution::RetainedRunOutcome};
pub(crate) use {worth_foundational::DeterminismContract, worth_foundational::ExecutionBudget, worth_foundational::ExecutionPosture, worth_foundational::ExecutionRequestPolicy, worth_foundational::PartitionIdentity};
pub(crate) use {worth_proof::CanonicalUniqueVec};

pub(crate) fn entries(
    ids: impl IntoIterator<Item = u64>,
) -> BTreeMap<PartitionIdentity, KeylessPartition<u64>> {
    ids.into_iter()
        .map(|id| {
            (
                worth_foundational::PartitionIdentity::new(id),
                KeylessPartition {
                    value: id,
                    kernel_scratch_bytes: 0,
                    max_result_bytes: 0,
                },
            )
        })
        .collect()
}