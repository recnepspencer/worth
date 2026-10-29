pub(crate) use {worth_store::physical_runtime::PhysicalPrefetchIntent, worth_store::physical_runtime::PhysicalPrefetchOutcome, worth_store::physical_runtime::PhysicalReadAheadFrameOutcome, worth_store::physical_runtime::PhysicalReadAheadIntent, worth_store::physical_runtime::PhysicalReadAheadOutcome, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalWorkOperationFamily, worth_store::physical_runtime::PhysicalWorkSignalFamily};

pub(crate) use {crate::physical_record_journeys::physical_work::speculative_residency::fixture::causal_record, crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate, crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::positioned_reads};

pub(crate) fn assert_kind_completion(
    before: worth_store_buffer_pool::PhysicalResidencyCounters,
    after: worth_store_buffer_pool::PhysicalResidencyCounters,
    kind: PhysicalSpeculativeWorkKind,
    operations: u64,
) {
    assert_eq!(
        after.speculative_attempts(kind),
        before.speculative_attempts(kind) + operations
    );
    assert_eq!(
        after.speculative_admissions(kind),
        before.speculative_admissions(kind) + operations
    );
    assert_eq!(
        after.speculative_completions(kind),
        before.speculative_completions(kind) + operations
    );
    assert_eq!(after.active_speculative_frames(kind), 0);
    assert_eq!(after.active_operation_bytes(), 0);
}