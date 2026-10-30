use {
    worth_store_testkit_store::durable_publication::root::publish_single,
    worth_store_testkit_store::physical_record_journeys::configuration,
    worth_store_testkit_store::physical_record_journeys::media,
    worth_store_testkit_store::physical_record_journeys::serving_from_initialization,
    worth_store_testkit_store::physical_record_journeys::success,
};

#[test]
fn serving_admission_reads_cross_the_frame_port() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("frame-mediated-admission");
    let (format, placement, access) = configuration();
    let initialized = success(initialize_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )
    }));
    assert_eq!(
        initialized
            .certification_frame_port_observer()
            .snapshot()
            .loads(),
        2
    );
    initialized.close();
    let opened = success(open_record_store!(media(&root), |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    }));
    assert_eq!(
        opened
            .certification_frame_port_observer()
            .snapshot()
            .loads(),
        3
    );
    opened.close();
}

#[test]
fn serving_observers_stale_before_media_release_can_block() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("terminating-store");
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(&root).unwrap(),
    )
    .unwrap();
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let admission = admission.with_fault_schedule(
        authority
            .schedule(Vec::new())
            .unwrap()
            .pause_before_lease_release(gate.clone()),
    );
    let worth_proof::TransitionOutcome::Success(media) =
        runtime.try_admit_filesystem_media(admission).into_raw()
    else {
        panic!("media admission must succeed")
    };
    let (format, placement, access) = configuration();
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        initialize_record_store!(media, |durability| {
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            )
        }),
    );
    let observer = serving.observer();
    let serving_generation = observer
        .acquisition_snapshot()
        .unwrap()
        .lifecycle_generation()
        .get();
    let close = std::thread::spawn(move || serving.close());
    gate.wait_until_reached();
    assert!(matches!(
        observer.acquisition_snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Stale {
            current_generation,
            ..
        }) if current_generation.get() == serving_generation + 1
    ));
    assert_eq!(observer.record_counters().owner_live(), 0);
    gate.release();
    assert_eq!(
        close.join().unwrap().records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::NoInspectionRequired
    );
    assert!(matches!(
        observer.acquisition_snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Closed {
            closed_generation,
            ..
        }) if closed_generation.get() == serving_generation + 2
    ));
}

#[test]
fn record_owner_propagates_through_every_lifecycle_boundary() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("close-store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let observer = serving.observer();
    let clone = observer.clone();
    assert_eq!(observer.record_counters().owner_live(), 1);

    let publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([148; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"lifecycle".as_slice()])
            .unwrap(),
    );
    let record = publication.settled_members()[0]
        .record_id(0)
        .expect("the completed singleton must expose its record identity");
    {
        let reader = serving.records().expect("read protection admission");
        assert_eq!(observer.record_counters().readers_live(), 1);
        let session = reader
            .open(
                record,
                worth_store::physical_runtime::RecordReadLimits::new(
                    worth_store::physical_runtime::RecordByteLimit::new(64).unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(observer.record_counters().read_sessions_live(), 1);
        drop(session);
        assert_eq!(observer.record_counters().read_sessions_live(), 0);
    }
    {
        let scan = serving
            .records()
            .expect("read protection admission")
            .scan(
                worth_store::physical_runtime::RecordScanRequest::from_start().with_batch_limit(
                    worth_store::physical_runtime::RecordCountLimit::new(1).unwrap(),
                ),
            )
            .unwrap();
        assert_eq!(observer.record_counters().readers_live(), 1);
        assert_eq!(observer.record_counters().scan_sessions_live(), 1);
        drop(scan);
    }
    assert_eq!(observer.record_counters().live_handles(), 0);
    let media_before = observer.media_counters();
    let closed = serving.close();
    assert_eq!(
        closed.records().posture(),
        worth_store::physical_runtime::RecordServingTerminalPosture::NoInspectionRequired
    );
    assert_eq!(
        closed.records().owner(),
        worth_store::physical_runtime::RecordServingOwnerDisposition::Released
    );
    assert_eq!(closed.records().counters().owner_live(), 0);
    assert_eq!(observer.record_counters().owner_live(), 0);
    assert_eq!(observer.record_counters().live_handles(), 0);
    let media_after = observer.media_counters();
    assert_eq!(
        media_after.ownership_releases(),
        media_before.ownership_releases() + 1
    );
    assert_eq!(media_after.live_directory_handles(), 0);
    for role in [
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
        worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        worth_store_physical_backend::MediaOperationRole::AtomicReplace,
        worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
    ] {
        assert_eq!(
            media_after.attempts_for(role),
            media_before.attempts_for(role)
        );
    }
    assert!(observer.acquisition_snapshot().is_err());
    drop((observer, clone));

    let abort_root = parent.path().join("abort-store");
    let aborting = serving_from_initialization(&abort_root);
    let abort_observer = aborting.observer();
    aborting.abort();
    assert_eq!(abort_observer.record_counters().owner_live(), 0);

    let drop_root = parent.path().join("drop-store");
    let dropped = serving_from_initialization(&drop_root);
    let drop_observer = dropped.observer();
    drop(dropped);
    assert_eq!(drop_observer.record_counters().owner_live(), 0);

    let panic_root = parent.path().join("panic-store");
    let panicking = serving_from_initialization(&panic_root);
    let panic_observer = panicking.observer();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _owned = panicking;
        panic!("controlled lifecycle panic");
    }));
    assert!(result.is_err());
    assert_eq!(panic_observer.record_counters().owner_live(), 0);
}

#[test]
fn consuming_record_admission_stales_media_observation_and_advances_lifecycle() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("phase-store");
    let (format, placement, access) = configuration();
    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let media_observer = media.observer();
    let media_generation = media_observer.snapshot().unwrap().generation();
    let serving = worth_store_testkit_store::physical_record_journeys::success(
        initialize_record_store!(media, |durability| {
            worth_store::physical_runtime::PhysicalRecordInitialization::new(
                format, placement, access, durability,
            )
        }),
    );

    assert!(matches!(
        media_observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Stale { .. })
    ));
    assert_eq!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .lifecycle_generation()
            .get(),
        media_generation.get() + 1
    );
    serving.close();

    let media = worth_store_testkit_store::physical_record_journeys::media(&root);
    let media_observer = media.observer();
    let media_generation = media_observer.snapshot().unwrap().generation();
    let serving = worth_store_testkit_store::physical_record_journeys::success(open_record_store!(
        media,
        |durability| {
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
        },
    ));
    assert!(matches!(
        media_observer.snapshot(),
        Err(worth_store::physical_runtime::ObservationError::Stale { .. })
    ));
    assert_eq!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .lifecycle_generation()
            .get(),
        media_generation.get() + 1
    );
    serving.close();
}

#[test]
fn record_observation_snapshot_has_one_acquisition_time_basis() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("observation-store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let before = serving.observer();
    let before_snapshot = before.acquisition_snapshot().unwrap();

    publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([149; 32]),
        worth_store::physical_runtime::RecordAppendBatch::try_from_iter([b"coherent".as_slice()])
            .unwrap(),
    );

    assert_eq!(before.acquisition_snapshot().unwrap(), before_snapshot);
    assert!(
        before.media_counters().completed_operations()
            > before_snapshot.media_counters().completed_operations()
    );
    let after_snapshot = serving.observer().acquisition_snapshot().unwrap();
    assert_eq!(
        after_snapshot.root_generation(),
        before_snapshot.root_generation() + 1
    );
    assert!(
        after_snapshot.media_counters().completed_operations()
            > before_snapshot.media_counters().completed_operations()
    );
    serving.close();
}

#[test]
fn serving_concurrency_contract_is_enforced_at_runtime_boundaries() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = serving_from_initialization(&root);
    let observer = serving.observer();
    let first = serving.records().unwrap();
    let second = serving.records().unwrap();
    assert_eq!(observer.record_counters().readers_live(), 2);
    assert_eq!(first.store_identity(), second.store_identity());
    drop((first, second));
    assert_eq!(observer.record_counters().readers_live(), 0);
    let child =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "second_owner",
            &root,
            None,
        );
    assert!(child.lines().any(|line| line == "C5_SECOND_OWNER denied"));
    serving.close();
}

#[path = "physical_residency.rs"]
mod physical_residency;
