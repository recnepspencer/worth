#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::maintenance_interference::sync::*;

use {
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::append,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::limits,
    worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement,
};

#[test]
fn unwinding_releases_file_sync_before_joining_the_scoped_worker() {
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let gate = admission.fault_schedule_authority().pause_gate();
    let completed = std::sync::atomic::AtomicBool::new(false);
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::thread::scope(|scope| {
            let _release = worth_store_testkit_store::physical_record_journeys::maintenance_interference::sync::FileSyncPauseRelease::new(&gate);
            scope.spawn(|| {
                gate.pause_for_certification();
                completed.store(true, std::sync::atomic::Ordering::Release);
            });
            gate.wait_until_reached();
            panic!("exercise file sync pause unwinding");
        });
    }));
    let panic = outcome.expect_err("scope must preserve the original panic");
    assert_eq!(
        panic.downcast_ref::<&str>(),
        Some(&"exercise file sync pause unwinding")
    );
    assert!(completed.load(std::sync::atomic::Ordering::Acquire));
}

#[test]
fn paused_file_sync_is_not_a_completed_barrier() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    initialize(&root).close();
    let admission = worth_store::physical_runtime::FilesystemMediaAdmission::production(
        worth_store_physical_backend::FilesystemAccessPosture::CoordinatedServiceAccount,
    );
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
                1,
                worth_store_physical_backend::MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = worth_store::physical_runtime::PhysicalStore::admit(
        worth_store::physical_runtime::PhysicalRuntimeAdmission::new(&root).unwrap(),
    )
    .unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let durability = worth_store_testkit_store::physical_record_journeys::durability(&media);
    let serving =
        worth_store_testkit_store::physical_record_journeys::success(media.open_record_store(
            worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability),
        ));
    activation.arm().unwrap();
    let finished = std::sync::atomic::AtomicBool::new(false);
    let id = std::thread::scope(|scope| {
        let append_thread = scope.spawn(|| {
            let id = append(&serving, placement(), 900, b"phase6-sync");
            finished.store(true, std::sync::atomic::Ordering::Release);
            id
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        let reached = gate.reached_context();
        let unfinished = !finished.load(std::sync::atomic::Ordering::Acquire);
        let counters = serving.media_counters();
        let attempts_while_held = counters
            .attempts_for(worth_store_physical_backend::MediaOperationRole::SynchronizeFileState);
        let barriers_while_held = counters.completed_operations_for(
            worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        );
        gate.release();
        let id = append_thread.join().unwrap();
        let reached = reached.expect("the append must reach a real file-state sync");
        assert_eq!(
            reached.role(),
            worth_store_physical_backend::MediaOperationRole::SynchronizeFileState
        );
        assert_eq!(reached.role_ordinal(), attempts_while_held);
        assert_eq!(
            barriers_while_held + 1,
            attempts_while_held,
            "the paused file-state sync is still in flight"
        );
        assert!(
            unfinished,
            "the append returned while its sync was still paused"
        );
        assert!(
            serving.media_counters().completed_operations_for(
                worth_store_physical_backend::MediaOperationRole::SynchronizeFileState
            ) > barriers_while_held
        );
        id
    });
    let mut session = serving.records().unwrap().open(id, limits()).unwrap();
    assert_eq!(
        session.next_chunk().unwrap().unwrap().bytes(),
        b"phase6-sync"
    );
    serving.close();
}
