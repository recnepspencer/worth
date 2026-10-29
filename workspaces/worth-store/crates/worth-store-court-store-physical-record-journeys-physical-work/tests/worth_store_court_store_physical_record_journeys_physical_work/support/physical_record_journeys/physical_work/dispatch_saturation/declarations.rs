pub(crate) use {std::time::Duration};

pub(crate) use {tempfile::tempdir};
pub(crate) use {worth_proof::NonEmpty};
pub(crate) use {worth_store::physical_runtime::ManagedPhysicalIntegrityScrubProgress, worth_store::physical_runtime::ManagedPhysicalIntegrityScrubRequest, worth_store::physical_runtime::PhysicalExecutorCommand, worth_store::physical_runtime::PhysicalIntegrityScrubDeferral, worth_store::physical_runtime::PhysicalIntegrityScrubReadDeferral, worth_store::physical_runtime::PhysicalIntegrityScrubTarget, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalRootPublicationPreparationOutcome, worth_store::physical_runtime::PhysicalRootPublicationWorkFailureCause, worth_store::physical_runtime::PhysicalRootReplacementFailureCause, worth_store::physical_runtime::PhysicalRootReplacementOutcome, worth_store::physical_runtime::PhysicalSchedulerDenial, worth_store::physical_runtime::PhysicalStoreCloseOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordSchedulerReservationDenial};
pub(crate) use {worth_store_io_scheduler::foreground_reservation::BandwidthToken, worth_store_io_scheduler::foreground_reservation::DirtyPageBudget, worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration, worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope, worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget, worth_store_io_scheduler::foreground_reservation::QueueSlot, worth_store_io_scheduler::foreground_reservation::WorkerPermit, worth_store_io_scheduler::foreground_reservation::WriteBackWindow};
pub(crate) use {worth_store_physical_backend::MediaOperationRole};
pub(crate) use {worth_store_physical_format::PhysicalArtifactReadTarget, worth_store_physical_format::PhysicalRecordFormatDeclaration, worth_store_physical_format::RecordArtifactFile};
pub(crate) use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange};

pub(crate) use {worth_store_testkit_store::physical_record_journeys::physical_work::executor::admitted_write, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::foreground_saturation_fixture, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_open_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::mutation_pair::whole_catalog_mutation_fixture, worth_store_testkit_store::physical_work::scheduler::policy_evidence::policy_receipt, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::ready_work, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::secure_demand, worth_store_testkit_store::physical_record_journeys::physical_work::scheduler::write_demand};

pub(crate) fn page_write_lane() -> ForegroundLaneDeclaration {
    worth_store_io_scheduler::foreground_reservation::ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(worth_store_io_scheduler::foreground_reservation::ForegroundLatencyEnvelope::bounded_interference(
            "physical-work-saturation-write",
            2,
        ))
        .with_budget(
            worth_store_io_scheduler::foreground_reservation::ForegroundResourceBudget::new()
                .with_queue_slots(worth_store_io_scheduler::foreground_reservation::QueueSlot::new(1).unwrap())
                .with_bandwidth(worth_store_io_scheduler::foreground_reservation::BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(worth_store_io_scheduler::foreground_reservation::WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(worth_store_io_scheduler::foreground_reservation::DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(worth_store_io_scheduler::foreground_reservation::WorkerPermit::new(1).unwrap()),
        )
}

pub(crate) fn execute_write(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    request: worth_store::physical_runtime::PhysicalMutationWorkRequest,
    bytes: &[u8],
) {
    let command =
        worth_store::physical_runtime::PhysicalExecutorCommand::exact_write(admitted_write(serving, request), bytes).unwrap();
    serving.execute_physical_work(command).unwrap();
}