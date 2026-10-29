pub(crate) use {std::sync::atomic::AtomicUsize, std::sync::atomic::Ordering};

pub(crate) use {worth_execution::ExecutionMap, worth_execution::MapDenial, worth_execution::MapKernelFailure, worth_execution::MapOutcome, worth_execution::MapPartition};
pub(crate) use {worth_foundational::PartitionIdentity};

pub(crate) fn partition(identity: u64, value: u64, reads: &[u64], writes: &[u64]) -> MapPartition<u64, u64> {
    MapPartition {
        identity: worth_foundational::PartitionIdentity::new(identity),
        value,
        read_keys: reads.to_vec(),
        write_keys: writes.to_vec(),
        kernel_scratch_bytes: 0,
        max_result_bytes: 8,
    }
}