#[allow(unused_imports)] use crate::physical_record_journeys::durability_admission::data_durability::concurrency::*;
use {std::sync::mpsc};
use {std::time::Duration};

use {worth_proof::NonEmpty, worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint};
use {worth_store::physical_runtime::PhysicalDataDispatchOutcome, worth_store::physical_runtime::PhysicalManifestCapacityTransition, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationOutcome, worth_store::physical_runtime::PhysicalMutationPreparationSuccess, worth_store::physical_runtime::PhysicalWalGroupAppendOutcome, worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome, worth_store::physical_runtime::PreparedPhysicalMutation, worth_store::physical_runtime::RecordAppendBatch, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::RecordWriteSource, worth_store::physical_runtime::RecordWriteSourceError};

use {worth_store_testkit_store::durable_publication::wal_data::prepare_single, worth_store_testkit_store::durable_publication::root::publish_single};
use {worth_store_testkit_store::physical_record_journeys::configuration, worth_store_testkit_store::physical_record_journeys::serving_from_initialization};
use {worth_store_testkit_store::physical_record_journeys::read_record};

#[test]
fn disjoint_data_effects_overlap_before_one_exact_group_root_publication() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let (_, placement, _) = configuration();
    let seed = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([150; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([SEED.as_slice()]).unwrap(),
    )
    .settled_members()[0]
        .record_id(0)
        .unwrap();
    let submission = serving.certification_record_submission();
    let left = prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([151; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([LEFT]).unwrap(),
    );
    let right = prepared(
        &submission,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([152; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([RIGHT]).unwrap(),
    );
    let appended = match submission.append_prepared_wal_group(worth_proof::NonEmpty::new(left, vec![right])) {
        worth_store::physical_runtime::PhysicalWalGroupAppendOutcome::Appended(appended) => appended,
        _ => panic!("the exact two-member WAL group must append"),
    };
    assert_disjoint_member_targets(&appended);
    let basis = appended.basis();
    let mut durable = match submission.synchronize_appended_wal_group(appended) {
        worth_store::physical_runtime::PhysicalWalGroupBarrierOutcome::Durable(durable) => durable.into_members().into_vec(),
        _ => panic!("the exact two-member WAL group must become durable"),
    };
    let right = durable.pop().expect("the group has a right member");
    let left = durable.pop().expect("the group has a left member");
    assert!(durable.is_empty());

    let dispatch_gate = serving.certification_pause_physical_execution_at(
        worth_store::physical_runtime::certification::CertificationPhysicalExecutionCheckpoint::BeforeBackendDispatch,
    );
    let left_submission = serving.certification_record_submission();
    let left_thread = std::thread::spawn(move || left_submission.dispatch_wal_durable_data(left));
    let right_submission = serving.certification_record_submission();
    let right_thread =
        std::thread::spawn(move || right_submission.dispatch_wal_durable_data(right));
    if !dispatch_gate.await_arrivals(2) {
        dispatch_gate.release();
        let left = left_thread.join().unwrap();
        let right = right_thread.join().unwrap();
        panic!(
            "both disjoint data effects must reach backend dispatch before either is released; \
             arrivals={}, left={}, right={}",
            dispatch_gate.arrival_count(),
            dispatch_label(&left),
            dispatch_label(&right),
        );
    }
    let seed_session = serving
        .records()
        .expect("read protection admission")
        .open(
            seed,
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(SEED.len() as u32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(seed_session, SEED.len()).0, SEED.as_slice());

    dispatch_gate.release();
    let left = dispatched(left_thread.join().unwrap());
    let right = dispatched(right_thread.join().unwrap());
    let completed =
        serving.certification_complete_dispatched_group(basis, worth_proof::NonEmpty::new(left, vec![right]));
    assert_eq!(completed.current_root().generation(), 3);
    assert_eq!(completed.settled_members().len(), 2);
    for (member, expected) in completed.settled_members().iter().zip([LEFT, RIGHT]) {
        let session = serving
            .records()
            .expect("read protection admission")
            .open(
                member.record_id(0).unwrap(),
                worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(expected.len() as u32).unwrap()),
            )
            .unwrap();
        assert_eq!(read_record(session, expected.len()).0, expected);
    }
    serving.close();
}

#[test]
fn paused_source_materialization_does_not_own_global_preparation_authority() {
    let parent = tempfile::tempdir().unwrap();
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let (entered_tx, entered_rx) = std::sync::mpsc::sync_channel(1);
    let (release_tx, release_rx) = std::sync::mpsc::sync_channel(1);
    let paused_submission = serving.certification_record_submission();
    let paused_thread = std::thread::spawn(move || {
        prepare_single(
            &paused_submission,
            placement,
            worth_store::physical_runtime::PhysicalManifestCapacityTransition::PreserveCurrent,
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([153; 32]),
            worth_store::physical_runtime::RecordAppendBatch::builder()
                .push_source(PausedInlineSource {
                    bytes: LEFT,
                    entered: entered_tx,
                    release: release_rx,
                    completed: false,
                })
                .build()
                .unwrap(),
        )
    });
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(3))
        .expect("the paused source must enter canonical materialization");
    let independent = prepared(
        &serving.certification_record_submission(),
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([154; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([RIGHT]).unwrap(),
    );
    release_tx.send(()).unwrap();
    let paused = into_prepared(paused_thread.join().unwrap());
    assert_ne!(
        paused.mutation_identity().operation_identity(),
        independent.mutation_identity().operation_identity()
    );
    drop((paused, independent));
    serving.close();
}
