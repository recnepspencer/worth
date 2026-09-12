use worth_store::physical_runtime::{
    FilesystemMediaAdmission, ObservationError, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRuntimeAdmission, PhysicalStore, RecordAppendBatch, RecordByteLimit, RecordCountLimit,
    RecordReadLimits, RecordScanRequest, RecordServingOwnerDisposition,
    RecordServingTerminalPosture,
};
use worth_store_physical_backend::{FilesystemAccessPosture, MediaOperationRole};

use worth_proof::TransitionOutcome;

use super::{
    configuration, durable_publication::publish_single, media, serving_from_initialization, success,
};

#[test]
fn serving_admission_reads_cross_the_frame_port() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("frame-mediated-admission");
    let (format, placement, access) = configuration();
    let initialized = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
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
        PhysicalRecordOpen::new(format, access, durability)
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
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let admission = admission.with_fault_schedule(
        authority
            .schedule(Vec::new())
            .unwrap()
            .pause_before_lease_release(gate.clone()),
    );
    let TransitionOutcome::Success(media) =
        runtime.try_admit_filesystem_media(admission).into_raw()
    else {
        panic!("media admission must succeed")
    };
    let (format, placement, access) = configuration();
    let serving = super::success(initialize_record_store!(media, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
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
        Err(ObservationError::Stale {
            current_generation,
            ..
        }) if current_generation.get() == serving_generation + 1
    ));
    assert_eq!(observer.record_counters().owner_live(), 0);
    gate.release();
    assert_eq!(
        close.join().unwrap().records().posture(),
        RecordServingTerminalPosture::NoInspectionRequired
    );
    assert!(matches!(
        observer.acquisition_snapshot(),
        Err(ObservationError::Closed {
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
        RecordAppendBatch::try_from_iter([b"lifecycle".as_slice()]).unwrap(),
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
                RecordReadLimits::new(RecordByteLimit::new(64).unwrap()),
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
                RecordScanRequest::from_start().with_batch_limit(RecordCountLimit::new(1).unwrap()),
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
        RecordServingTerminalPosture::NoInspectionRequired
    );
    assert_eq!(
        closed.records().owner(),
        RecordServingOwnerDisposition::Released
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
        MediaOperationRole::PositionedWrite,
        MediaOperationRole::SynchronizeFileState,
        MediaOperationRole::AtomicReplace,
        MediaOperationRole::SynchronizeDirectoryPublication,
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
    let media = super::media(&root);
    let media_observer = media.observer();
    let media_generation = media_observer.snapshot().unwrap().generation();
    let serving = super::success(initialize_record_store!(media, |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));

    assert!(matches!(
        media_observer.snapshot(),
        Err(ObservationError::Stale { .. })
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

    let media = super::media(&root);
    let media_observer = media.observer();
    let media_generation = media_observer.snapshot().unwrap().generation();
    let serving = super::success(open_record_store!(media, |durability| {
        worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
    },));
    assert!(matches!(
        media_observer.snapshot(),
        Err(ObservationError::Stale { .. })
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
        RecordAppendBatch::try_from_iter([b"coherent".as_slice()]).unwrap(),
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
    let child = super::child_process::run_child("second_owner", &root, None);
    assert!(child.lines().any(|line| line == "C5_SECOND_OWNER denied"));
    serving.close();
}

#[test]
fn physical_residency_serves_real_reads_and_candidate_writes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (_, placement, _) = configuration();
    let serving = serving_from_initialization(&root);
    let first_publication = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([150; 32]),
        RecordAppendBatch::try_from_iter([b"direct".as_slice()]).unwrap(),
    );
    let first = first_publication.settled_members()[0].record_id(0).unwrap();

    let counters = serving.certification_frame_port_observer();
    let mut read = serving
        .records()
        .expect("read protection admission")
        .open(
            first,
            RecordReadLimits::new(RecordByteLimit::new(64).unwrap()),
        )
        .unwrap();
    let mut bytes = [0_u8; 6];
    assert_eq!(read.read_next(&mut bytes).unwrap(), bytes.len());
    assert_eq!(&bytes, b"direct");
    drop(read);
    let second = publish_single(
        &serving,
        placement,
        worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([151; 32]),
        RecordAppendBatch::try_from_iter([b"wrapped".as_slice()]).unwrap(),
    );
    assert_eq!(second.current_root().generation(), 3);
    let mut retained_read = serving
        .records()
        .expect("read protection admission")
        .open(
            second.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(64).unwrap()),
        )
        .unwrap();
    let mut retained_bytes = [0_u8; 7];
    assert_eq!(
        retained_read.read_next(&mut retained_bytes).unwrap(),
        retained_bytes.len()
    );
    assert_eq!(&retained_bytes, b"wrapped");
    drop(retained_read);
    let snapshot = counters.snapshot();
    assert!(snapshot.loads() > 0);
    assert!(snapshot.residency_faults() > 0);
    assert!(snapshot.residency_hits() > 0);
    assert_eq!(snapshot.writebacks(), 0);
    assert_eq!(
        snapshot.candidate_publications(),
        snapshot.candidate_frames()
    );
    assert!(snapshot.loads() > 0);
    assert!(snapshot.candidate_submissions() >= 1);
    assert!(snapshot.candidate_frames() >= 5);
    assert!(snapshot.candidate_bytes() > b"wrapped".len() as u64);
    assert_eq!(
        snapshot.declared_candidate_frames(),
        snapshot.candidate_frames()
    );
    assert_eq!(
        snapshot.declared_candidate_bytes(),
        snapshot.candidate_bytes()
    );
    assert_eq!(snapshot.wrapper_frames(), 0);
    assert!(snapshot.peak_retained_candidate_frames() >= 1);
    let shutdown = serving.close();
    assert!(!shutdown.residency().requires_inspection());
}
