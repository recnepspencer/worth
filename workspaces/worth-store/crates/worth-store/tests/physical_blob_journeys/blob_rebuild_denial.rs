use std::num::NonZeroU64;

use worth_store::physical_runtime::{
    BlobCheckpointLimit, BlobIngestDeclaration, BlobReadLimits, LayoutRebuildFailure,
    LayoutRebuildLimits, PhysicalMutationDeadline, PhysicalOperationAllocationScope,
    PhysicalResidencyDimension, PhysicalWorkCounterStage, RecordReadDenial, RecordReadWorkDenial,
    RecordScanDenial,
};
use worth_store_blob_chunks::BlobChunkSize;
use worth_store_contracts::DurableArtifactFamilyId;
use worth_store_io_scheduler::foreground_reservation::{
    ForegroundLaneDeclaration, ForegroundLatencyEnvelope, ForegroundResourceBudget, QueueSlot,
    WorkerPermit,
};

use super::fixture::{
    admitted_blob_scope, placement, serving_from_initialization, serving_from_open,
};

#[test]
fn denied_cold_blob_rebuild_releases_maintenance_allocation_and_scheduler_capacity() {
    let directory = tempfile::tempdir().unwrap();
    let scope = admitted_blob_scope("c11.blob.rebuild.denial.scope");
    let limits = BlobReadLimits::new(NonZeroU64::new(64).unwrap());
    let serving = serving_from_initialization(directory.path());
    let blobs = serving.blobs().unwrap();
    let object = blobs.issue_object_id(limits).unwrap();
    let declaration = BlobIngestDeclaration::new(
        object,
        BlobChunkSize::from_bytes(64 << 10).unwrap(),
        2,
        &scope,
        BlobCheckpointLimit::bounded_horizon(64).unwrap(),
        PhysicalMutationDeadline::after_milliseconds(120_000).unwrap(),
    )
    .unwrap();
    let mut ingest = blobs
        .begin_ingest(declaration, placement(), 1, limits)
        .unwrap();
    ingest.push(&[0x7e]).unwrap();
    ingest.push(&[0x7f]).unwrap();
    let published = ingest.finish().unwrap();
    drop(blobs);
    serving.close();

    let serving = serving_from_open(directory.path());
    let blobs = serving.blobs().unwrap();
    let slots = serving
        .physical_scheduler_capacity()
        .configured()
        .queue_slots();
    let blocker = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "blob-rebuild-denial-blocker",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(slots / 2 + 1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    let blocker = serving
        .reserve_physical_scheduler_foreground(blocker)
        .expect("foreground reservation can hold the background share");
    let capacity_held = serving.physical_scheduler_capacity();
    let work_before = serving.physical_work_counters();
    let allocation_before = serving.residency_observation().allocations();

    let rebuild_limits = LayoutRebuildLimits::new(
        NonZeroU64::new(1_000).unwrap(),
        NonZeroU64::new(1_000).unwrap(),
    );
    let denied = serving.layouts().unwrap().rebuild(
        DurableArtifactFamilyId::BlobCatalog,
        rebuild_limits,
        placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    assert!(
        matches!(
            &denied,
            Err(LayoutRebuildFailure::Scan(error))
                if matches!(
                    error.denial(),
                    RecordScanDenial::RecordRead(RecordReadDenial::PhysicalWork(
                        RecordReadWorkDenial::SchedulerRejected
                    ))
                )
        ),
        "the cold selected-authority rebuild must reach the real Rebuild scheduler: {denied:?}"
    );
    let work_after = serving.physical_work_counters();
    for stage in [
        PhysicalWorkCounterStage::Declared,
        PhysicalWorkCounterStage::Ready,
        PhysicalWorkCounterStage::Queued,
        PhysicalWorkCounterStage::Dispatched,
        PhysicalWorkCounterStage::Settling,
    ] {
        assert_eq!(
            work_after.total(stage),
            work_before.total(stage),
            "scheduler denial cannot execute physical work at {stage:?}"
        );
    }
    let allocation_after = serving.residency_observation();
    for scope in [
        PhysicalOperationAllocationScope::Blob,
        PhysicalOperationAllocationScope::Maintenance,
    ] {
        let dimension = PhysicalResidencyDimension::OperationScope(scope);
        let before = allocation_before.for_dimension(dimension);
        let after = allocation_after.allocations().for_dimension(dimension);
        // C11 rebuild owns Maintenance scratch, not a foreground Blob read
        // allocation. Every admitted charge must be released on denial.
        let admitted = after.admissions() - before.admissions();
        match scope {
            PhysicalOperationAllocationScope::Blob => assert_eq!(admitted, 0),
            PhysicalOperationAllocationScope::Maintenance => assert!(admitted > 0),
            _ => unreachable!("only the asserted scopes are iterated"),
        }
        assert_eq!(after.releases(), before.releases() + admitted, "{scope:?}");
        assert_eq!(after.active_units(), before.active_units(), "{scope:?}");
        assert_eq!(
            allocation_after
                .counters()
                .active_operation_bytes_for(scope),
            0,
            "{scope:?} allocation must not survive denied Rebuild admission"
        );
    }
    let capacity_after_denial = serving.physical_scheduler_capacity();
    assert_eq!(capacity_after_denial.available(), capacity_held.available());
    assert_eq!(capacity_after_denial.active_reservations(), 1);
    drop(blocker);
    let capacity_released = serving.physical_scheduler_capacity();
    assert_eq!(
        capacity_released.available(),
        capacity_released.configured()
    );
    assert_eq!(capacity_released.active_reservations(), 0);

    let rebuilt = serving.layouts().unwrap().rebuild(
        DurableArtifactFamilyId::BlobCatalog,
        rebuild_limits,
        placement(),
        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
    );
    assert!(
        rebuilt.is_ok(),
        "a denied synchronous rebuild must not retain fairness head: {rebuilt:?}"
    );
    let resolved = blobs
        .resolve_publication(
            object.bytes(),
            published.generation().sequence(),
            &scope,
            limits,
        )
        .expect("a denied synchronous attempt must not retain the fairness head");
    assert_eq!(resolved, published);
    let mut read = blobs.read(resolved, &scope, 0, 2, limits).unwrap();
    let mut bytes = [0_u8; 2];
    assert_eq!(read.read_next(&mut bytes).unwrap(), 2);
    assert_eq!(bytes, [0x7e, 0x7f]);
    assert_eq!(read.read_next(&mut bytes).unwrap(), 0);
    drop(read);
    drop(blobs);
    serving.close();
}
