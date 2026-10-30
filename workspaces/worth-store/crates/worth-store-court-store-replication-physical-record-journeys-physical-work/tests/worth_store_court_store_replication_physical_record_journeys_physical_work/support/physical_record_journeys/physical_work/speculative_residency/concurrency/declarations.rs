pub(crate) use {
    worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint,
    worth_store::physical_runtime::PhysicalOperationAllocationScope,
    worth_store::physical_runtime::PhysicalPrefetchIntent,
    worth_store::physical_runtime::PhysicalPrefetchOutcome,
    worth_store::physical_runtime::PhysicalReadAheadFrameOutcome,
    worth_store::physical_runtime::PhysicalReadAheadIntent,
    worth_store::physical_runtime::PhysicalReadAheadOutcome,
    worth_store::physical_runtime::PhysicalResidencyDimension,
    worth_store::physical_runtime::PhysicalResidencyRetryPosture,
    worth_store::physical_runtime::PhysicalSpeculativeWorkKind,
};

pub(crate) use {
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::causal_record,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store,
    crate::physical_record_journeys::physical_work::speculative_residency::fixture::positioned_reads,
};

pub(crate) fn await_coalesced_waiter(
    residency: &worth_store::physical_runtime::PhysicalResidencyCertification,
    before: u64,
) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while residency.counters().coalesced_waiters() == before {
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}
