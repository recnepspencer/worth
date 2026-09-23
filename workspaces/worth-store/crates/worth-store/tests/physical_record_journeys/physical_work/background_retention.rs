use super::{
    executor::admitted_write,
    fixture::{foreground_saturation_fixture, serving_from_initialization_with_work_profile},
};
use tempfile::tempdir;
use worth_store::physical_runtime::{
    PhysicalExecutorCommand, PhysicalStoreCloseOutcome, RecordSchedulerReservationDenial,
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
