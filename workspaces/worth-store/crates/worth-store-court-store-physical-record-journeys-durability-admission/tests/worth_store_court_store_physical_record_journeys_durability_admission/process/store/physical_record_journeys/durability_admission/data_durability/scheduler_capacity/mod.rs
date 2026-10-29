#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::data_durability::scheduler_capacity::*;
use {std::sync::mpsc};
use {std::time::Duration};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
use {worth_store::physical_runtime::PhysicalDataDispatchFailureCause, worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalRecordMutationFailureCause, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PhysicalWorkCapacity, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::WalDurablePhysicalMutation};
use {worth_store_io_scheduler::foreground_reservation::ForegroundReservationAdmissionDenial, worth_store_io_scheduler::foreground_reservation::ForegroundReservationResourceShortfall, worth_store_io_scheduler::foreground_reservation::PhysicalInstanceForegroundAdmissionDenial};

use {worth_store_testkit_store::physical_record_journeys::configuration};
use {worth_store_testkit_store::durable_publication::wal_data::prepare_single, worth_store_testkit_store::durable_publication::root::publish_single};
use {worth_store_testkit_store::physical_record_journeys::physical_work::fixture::runtime::serving_from_initialization_with_work_profile, worth_store_testkit_store::physical_record_journeys::physical_work::fixture::semantic_work::work_fixture};

#[test]
fn wal_durable_data_retry_preserves_identity_across_exact_scheduler_exhaustion() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (profile, _, _) = work_fixture();
    let capacity = worth_store::physical_runtime::PhysicalWorkCapacity::new(
        8,
        256,
        1_024,
        PAGE_GRANT_BYTES as usize,
        SCHEDULER_BANDWIDTH_BYTES as usize,
    )
    .unwrap();
    let serving =
        serving_from_initialization_with_work_profile(&root, profile.with_capacity(capacity));
    let (_, placement, _) = configuration();
    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([160; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([SEED.as_slice()]).unwrap(),
    );

    let submission = serving.certification_record_submission();
    let left = prepared(&submission, placement, [161; 32], LEFT);
    let right = prepared(&submission, placement, [162; 32], RIGHT);
    let retry = prepared(&submission, placement, [163; 32], RETRY);
    let appended =
        match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(left, vec![right, retry])) {
            worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
            _ => panic!("the exact three-member WAL group must append"),
        };
    let basis = appended.basis();
    let mut durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable.into_members().into_vec(),
        _ => panic!("the exact three-member WAL group must become durable"),
    };
    let retry = durable.pop().expect("the group has a retry member");
    let right = durable.pop().expect("the group has a right member");
    let left = durable.pop().expect("the group has a left member");
    assert!(durable.is_empty());
    let retry_identity = retry.mutation_identity();

    let gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let left_submission = serving.certification_record_submission();
    let left_thread = std::thread::spawn(move || left_submission.dispatch_wal_durable_data(left));
    let right_submission = serving.certification_record_submission();
    let right_thread =
        std::thread::spawn(move || right_submission.dispatch_wal_durable_data(right));
    if !gate.await_arrivals(2) {
        gate.release();
        let _ = left_thread.join();
        let _ = right_thread.join();
        panic!("two data effects must hold both scheduler reservations");
    }
    let saturated = serving.physical_scheduler_capacity();
    assert_eq!(saturated.active_reservations(), 2);
    assert_eq!(
        saturated.available().bandwidth_tokens(),
        BANDWIDTH_AFTER_TWO_DATA_EFFECTS
    );

    let media_before_denial = serving.media_counters();
    let (denied_tx, denied_rx) = std::sync::mpsc::sync_channel(1);
    let retry_submission = serving.certification_record_submission();
    let denied_thread = std::thread::spawn(move || {
        denied_tx
            .send(retry_submission.dispatch_wal_durable_data(retry))
            .unwrap();
    });
    let denied = match denied_rx.recv_timeout(std::time::Duration::from_secs(3)) {
        Ok(outcome) => outcome,
        Err(_) => {
            gate.release();
            let _ = left_thread.join();
            let _ = right_thread.join();
            let _ = denied_thread.join();
            panic!("the third data effect did not receive a pre-dispatch scheduler denial");
        }
    };
    denied_thread.join().unwrap();
    assert_eq!(serving.media_counters(), media_before_denial);
    let retry = exact_bandwidth_denial(denied);
    assert_eq!(retry.mutation_identity(), retry_identity);
    let denied_snapshot = serving.physical_scheduler_capacity();
    assert_eq!(denied_snapshot.active_reservations(), 2);
    assert_eq!(denied_snapshot.denied_reservations(), 1);

    gate.release();
    let left = dispatched(left_thread.join().unwrap());
    let right = dispatched(right_thread.join().unwrap());
    let released = serving.physical_scheduler_capacity();
    assert_eq!(released.active_reservations(), 0);
    assert_eq!(released.available(), released.configured());

    let retry = dispatched(
        serving
            .certification_record_submission()
            .dispatch_wal_durable_data(retry),
    );
    let completed = serving
        .certification_complete_dispatched_group(basis, worth_proof::NonEmpty::new(left, vec![right, retry]));
    assert_eq!(completed.settled_members().len(), 3);
    let final_snapshot = serving.physical_scheduler_capacity();
    assert_eq!(final_snapshot.active_reservations(), 0);
    assert_eq!(final_snapshot.available(), final_snapshot.configured());
    serving.close();
}
