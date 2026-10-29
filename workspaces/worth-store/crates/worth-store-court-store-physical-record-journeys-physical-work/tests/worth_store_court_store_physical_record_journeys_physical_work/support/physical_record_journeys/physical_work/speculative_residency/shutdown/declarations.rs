pub(crate) use {std::time::Duration, std::time::Instant};

pub(crate) use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint, worth_store::physical_runtime::PhysicalPrefetchIntent, worth_store::physical_runtime::PhysicalPrefetchOutcome, worth_store::physical_runtime::PhysicalReadAheadFrameOutcome, worth_store::physical_runtime::PhysicalReadAheadIntent, worth_store::physical_runtime::PhysicalReadAheadOutcome, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalStoreCloseObservation, worth_store::physical_runtime::PhysicalStoreClosePhase, worth_store::physical_runtime::PhysicalWritebackExecution};
pub(crate) use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};

pub(crate) use {crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate, crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store_with_writebehind};

pub(crate) fn await_close_blocked_on_possible_effect(progress: &PhysicalStoreCloseObservation) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped) && std::time::Instant::now() < deadline
    {
        std::thread::yield_now();
    }
    assert!(
        progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::AdmissionStopped),
        "close never entered the shutdown protocol"
    );
    assert!(
        !progress.reached(worth_store::physical_runtime::PhysicalStoreClosePhase::SafeCancellationComplete),
        "close classified safe cancellation before the live possible effect settled"
    );
}