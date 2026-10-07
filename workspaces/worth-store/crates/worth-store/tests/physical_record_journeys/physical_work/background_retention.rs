use super::{
    executor::admitted_write,
    fixture::{foreground_saturation_fixture, serving_from_initialization_with_work_profile},
};
use tempfile::tempdir;
use worth_store::physical_runtime::{
    PhysicalExecutorCommand, PhysicalStoreCloseOutcome, PhysicalWorkCounterStage,
    RecordSchedulerReservationDenial,
};
use worth_store_io_scheduler::foreground_reservation::{
    BandwidthToken, DirtyPageBudget, ForegroundLaneDeclaration, ForegroundLatencyEnvelope,
    ForegroundResourceBudget, QueueSlot, WorkerPermit, WriteBackWindow,
};

fn page_write_lane() -> ForegroundLaneDeclaration {
    ForegroundLaneDeclaration::ordinary_page_write()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "retained-background-head",
            2,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(1).unwrap())
                .with_bandwidth(BandwidthToken::bytes(4_096).unwrap())
                .with_write_back(WriteBackWindow::pages(1).unwrap())
                .with_dirty_pages(DirtyPageBudget::pages(1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        )
}

#[test]
fn yielded_checkpoint_and_reclamation_heads_block_foreground_until_admit_or_cancel() {
    let root = tempdir().unwrap();
    let (profile, writes) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let [first, second, third, fourth] = writes;
    for (request, bytes) in [first, second, third].into_iter().zip([
        b"write-01".as_slice(),
        b"write-02".as_slice(),
        b"write-03".as_slice(),
    ]) {
        let command =
            PhysicalExecutorCommand::exact_write(admitted_write(&serving, request), bytes).unwrap();
        serving.execute_physical_work(command).unwrap();
    }

    assert!(serving.certification_checkpoint_work_yields_for_foreground_pressure());
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write_lane()),
        Err(RecordSchedulerReservationDenial::OwedBackgroundTurn)
    ));
    serving.certification_cancel_checkpoint_background_head();
    serving
        .reserve_physical_scheduler_foreground(page_write_lane())
        .expect("cancelling the checkpoint head releases the owed turn");

    assert!(serving.certification_reclamation_work_yields_for_foreground_pressure());
    assert!(matches!(
        serving.reserve_physical_scheduler_foreground(page_write_lane()),
        Err(RecordSchedulerReservationDenial::OwedBackgroundTurn)
    ));
    assert!(serving.certification_reclamation_work_admits_when_foreground_is_idle());
    serving
        .reserve_physical_scheduler_foreground(page_write_lane())
        .expect("an admitted reclamation quantum releases the owed turn");

    let command = PhysicalExecutorCommand::exact_write(
        admitted_write(&serving, fourth),
        b"write-04".as_slice(),
    )
    .unwrap();
    serving.execute_physical_work(command).unwrap();
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn denied_synchronous_blob_quantum_releases_its_fairness_head() {
    let root = tempdir().unwrap();
    let (profile, _) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    assert!(serving.certification_reject_oversized_blob_ingest_quantum());
    for _ in 0..4 {
        serving
            .reserve_physical_scheduler_foreground(page_write_lane())
            .expect("a denied blob attempt must not retain an owed background turn");
    }
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn denied_synchronous_blob_reclaim_quantum_releases_its_fairness_head() {
    let root = tempdir().unwrap();
    let (profile, _) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    assert!(serving.certification_reject_oversized_blob_reclaim_quantum());
    for _ in 0..4 {
        serving
            .reserve_physical_scheduler_foreground(page_write_lane())
            .expect("a denied reclaim attempt must not retain an owed background turn");
    }
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn foreground_capacity_defers_reclaim_without_consuming_the_foreground_floor() {
    let root = tempdir().unwrap();
    let (profile, _) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let configured = serving
        .physical_scheduler_capacity()
        .configured()
        .queue_slots();
    let held = ForegroundLaneDeclaration::artifact_metadata_read()
        .with_latency_envelope(ForegroundLatencyEnvelope::bounded_interference(
            "reclaim-capacity-blocker",
            1,
        ))
        .with_budget(
            ForegroundResourceBudget::new()
                .with_queue_slots(QueueSlot::new(configured / 2 + 1).unwrap())
                .with_worker_permits(WorkerPermit::new(1).unwrap()),
        );
    let held = serving
        .reserve_physical_scheduler_foreground(held)
        .expect("foreground can consume the background share");
    let before_media = serving.media_counters();
    let before_work = serving.physical_work_counters();
    assert!(serving.certification_blob_reclaim_quantum_denied_by_capacity(4_096));
    assert_eq!(serving.media_counters(), before_media);
    assert_eq!(serving.physical_work_counters(), before_work);
    let ordinary = serving
        .reserve_physical_scheduler_foreground(page_write_lane())
        .expect("a deferred reclaim leaves the foreground floor available");
    drop(ordinary);
    drop(held);
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}

#[test]
fn denied_synchronous_rebuild_quantum_releases_its_fairness_head() {
    let root = tempdir().unwrap();
    let (profile, _) = foreground_saturation_fixture();
    let serving = serving_from_initialization_with_work_profile(root.path(), profile);
    let (denied, [before_work, after_work], [before_capacity, after_capacity]) =
        serving.certification_reject_oversized_rebuild_read_quantum();
    assert!(denied);
    assert_eq!(
        before_work, after_work,
        "denial must precede work declaration"
    );
    assert_eq!(after_work.total(PhysicalWorkCounterStage::Declared), 0);
    assert_eq!(before_capacity.available(), after_capacity.available());
    assert_eq!(before_capacity.active_reservations(), 0);
    assert_eq!(after_capacity.active_reservations(), 0);
    for _ in 0..4 {
        serving
            .reserve_physical_scheduler_foreground(page_write_lane())
            .expect("a denied Rebuild attempt must not retain an owed background turn");
    }
    assert!(matches!(
        serving.close_plan().execute(),
        PhysicalStoreCloseOutcome::Closed { .. }
    ));
}
