// A Native consumer measures backend admission with its permitted allocator probe.
pub(crate) use {stats_alloc::Region, stats_alloc::StatsAlloc, stats_alloc::INSTRUMENTED_SYSTEM};
pub(crate) use {worth_execution::KeyedEditDenial, worth_execution::KeyedItem, worth_execution::KeyedPartitioner, worth_execution::PartitionItemId};
pub(crate) use {worth_foundational::PartitionIdentity};

#[global_allocator]
pub(crate) static ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;