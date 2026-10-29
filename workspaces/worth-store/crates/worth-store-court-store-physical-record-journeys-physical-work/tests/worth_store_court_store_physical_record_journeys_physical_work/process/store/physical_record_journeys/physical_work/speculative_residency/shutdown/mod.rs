#[allow(unused_imports)] use crate::physical_record_journeys::physical_work::speculative_residency::shutdown::*;
use {std::time::Duration, std::time::Instant};

use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint, worth_store::physical_runtime::PhysicalPrefetchIntent, worth_store::physical_runtime::PhysicalPrefetchOutcome, worth_store::physical_runtime::PhysicalReadAheadFrameOutcome, worth_store::physical_runtime::PhysicalReadAheadIntent, worth_store::physical_runtime::PhysicalReadAheadOutcome, worth_store::physical_runtime::PhysicalSpeculativeWorkKind, worth_store::physical_runtime::PhysicalStoreCloseObservation, worth_store::physical_runtime::PhysicalStoreClosePhase, worth_store::physical_runtime::PhysicalWritebackExecution};
use {worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement};

use {crate::physical_record_journeys::physical_work::speculative_residency::fixture::coordinate, crate::physical_record_journeys::physical_work::speculative_residency::fixture::initialize_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store, crate::physical_record_journeys::physical_work::speculative_residency::fixture::open_store_with_writebehind};

#[test]
fn close_joins_paused_speculation_and_reports_no_residue() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("speculative-close");
    initialize_store(&root);
    let serving = open_store(&root, 2, 2);
    let residency = serving.certification_physical_residency();
    residency.drain_unpinned_clean_frames();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement,
    );
    let owner_residency = residency.clone();
    let owner = std::thread::spawn(move || {
        owner_residency.prefetch(worth_store::physical_runtime::PhysicalPrefetchIntent::new(coordinate(0)))
    });
    assert!(gate.await_arrival());

    let close_plan = serving.close_plan();
    let close_progress = close_plan.observation();
    let close = std::thread::spawn(move || close_plan.execute());
    await_close_blocked_on_possible_effect(&close_progress);
    gate.release();
    let owner_outcome = owner.join().unwrap();
    assert!(
        matches!(owner_outcome, worth_store::physical_runtime::PhysicalPrefetchOutcome::Loaded { .. }),
        "paused speculative owner did not settle after dispatch: {owner_outcome:?}"
    );
    let closed = close.join().unwrap().into_shutdown();

    assert!(!closed.residency().requires_inspection());
    let counters = closed.residency().counters();
    assert_eq!(
        counters.active_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::Prefetch),
        0
    );
    assert_eq!(counters.active_operation_bytes(), 0);
}

#[test]
fn close_joins_paused_read_ahead_possible_effect_and_reports_no_residue() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("read-ahead-close");
    initialize_store(&root);
    let serving = open_store(&root, 2, 2);
    let residency = serving.certification_physical_residency();
    residency.drain_unpinned_clean_frames();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterReadBeforeSchedulerSettlement,
    );
    let coordinates = [coordinate(0)];
    let owner_residency = residency.clone();
    let owner = std::thread::spawn(move || {
        owner_residency.read_ahead(worth_store::physical_runtime::PhysicalReadAheadIntent::new(&coordinates).unwrap())
    });
    assert!(gate.await_arrival());

    let close_plan = serving.close_plan();
    let close_progress = close_plan.observation();
    let close = std::thread::spawn(move || close_plan.execute());
    await_close_blocked_on_possible_effect(&close_progress);
    gate.release();
    let batch = match owner.join().unwrap() {
        worth_store::physical_runtime::PhysicalReadAheadOutcome::Complete(batch) => batch,
        outcome => panic!("paused read-ahead did not settle after dispatch: {outcome:?}"),
    };
    assert!(matches!(
        batch.frames(),
        [worth_store::physical_runtime::PhysicalReadAheadFrameOutcome::Loaded { .. }]
    ));
    let closed = close.join().unwrap().into_shutdown();

    assert!(!closed.residency().requires_inspection());
    let counters = closed.residency().counters();
    assert_eq!(
        counters.active_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::ReadAhead),
        0
    );
    assert_eq!(counters.active_operation_bytes(), 0);
}

#[test]
fn close_joins_paused_writebehind_possible_effect_and_reports_no_residue() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("writebehind-close");
    initialize_store(&root);
    let serving = open_store_with_writebehind(&root, 2, 2, 1);
    let residency = serving.certification_physical_residency();
    let dirty = residency
        .admit_dirty_frame(residency.pin_exact(coordinate(0)).unwrap(), |_, target| {
            target.fill(0x73);
        })
        .unwrap();
    let admitted = residency
        .admit_writeback(
            residency
                .request_writeback(
                    residency
                        .prepare_writeback(
                            dirty,
                            worth_store_physical_backend::ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
                        )
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::AfterResidencyWriteBeforeSchedulerSettlement,
    );
    let owner_residency = residency.clone();
    let owner = std::thread::spawn(move || owner_residency.execute_writeback(admitted));
    assert!(gate.await_arrival());

    let close_plan = serving.close_plan();
    let close_progress = close_plan.observation();
    let close = std::thread::spawn(move || close_plan.execute());
    await_close_blocked_on_possible_effect(&close_progress);
    gate.release();
    assert!(matches!(
        owner.join().unwrap().unwrap(),
        worth_store::physical_runtime::PhysicalWritebackExecution::Clean(_)
    ));
    let closed = close.join().unwrap().into_shutdown();

    assert!(!closed.residency().requires_inspection());
    let counters = closed.residency().counters();
    assert_eq!(
        counters.active_speculative_frames(worth_store::physical_runtime::PhysicalSpeculativeWorkKind::WriteBehind),
        0
    );
    assert_eq!(counters.dirty_frames(), 0);
    assert_eq!(counters.active_operation_bytes(), 0);
}
