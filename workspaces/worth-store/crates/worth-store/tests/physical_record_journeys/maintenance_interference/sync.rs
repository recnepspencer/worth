use std::sync::atomic::{AtomicBool, Ordering};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, FilesystemMediaAdmission, PhysicalReadProtectionPolicy,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalPolicy,
    PhysicalWorkProfileDeclaration, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::{CertificationMediaFaultActivation, MediaPauseGate};
use worth_store_physical_backend::{
    FilesystemAccessPosture, MediaFaultDirective, MediaOperationRole,
};

use super::{append, initialize, limits, placement};

#[test]
fn paused_file_sync_is_not_a_completed_barrier() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    initialize(&root).close();
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(&root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let durability = super::super::durability(&media);
    let serving = super::super::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, durability)),
    );
    activation.arm().unwrap();
    let finished = AtomicBool::new(false);
    let id = std::thread::scope(|scope| {
        let append_thread = scope.spawn(|| {
            let id = append(&serving, placement(), 900, b"phase6-sync");
            finished.store(true, Ordering::Release);
            id
        });
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while gate.reached_context().is_none() && std::time::Instant::now() < deadline {
            std::thread::yield_now();
        }
        let reached = gate.reached_context();
        let unfinished = !finished.load(Ordering::Acquire);
        let counters = serving.media_counters();
        let attempts_while_held = counters.attempts_for(MediaOperationRole::SynchronizeFileState);
        let barriers_while_held =
            counters.completed_operations_for(MediaOperationRole::SynchronizeFileState);
        gate.release();
        let id = append_thread.join().unwrap();
        let reached = reached.expect("the append must reach a real file-state sync");
        assert_eq!(reached.role(), MediaOperationRole::SynchronizeFileState);
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
            serving
                .media_counters()
                .completed_operations_for(MediaOperationRole::SynchronizeFileState)
                > barriers_while_held
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

pub(super) fn open_with_file_sync_pause(
    root: &std::path::Path,
    profile: PhysicalWorkProfileDeclaration,
) -> (
    worth_store::physical_runtime::ServingPhysicalRuntime,
    MediaPauseGate,
    CertificationMediaFaultActivation,
) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let durability = super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    let initialization =
        PhysicalRecordInitialization::new(format, super::placement_for(format), access, durability)
            .with_residency_policy(super::residency(format, super::RESIDENT_BYTES))
            .with_read_protection_policy(PhysicalReadProtectionPolicy::default())
            .with_physical_work_profile(profile);
    let serving = super::super::success(media.initialize_record_store(initialization));
    (serving, gate, activation)
}

pub(super) fn reopen_with_file_sync_pause(
    root: &std::path::Path,
    profile: PhysicalWorkProfileDeclaration,
) -> (
    worth_store::physical_runtime::ServingPhysicalRuntime,
    MediaPauseGate,
    CertificationMediaFaultActivation,
) {
    let admission =
        FilesystemMediaAdmission::production(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let gate = authority.pause_gate();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(
                MediaOperationRole::SynchronizeFileState,
                1,
                MediaFaultDirective::PauseBefore(gate.clone()),
            )
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("faulted media should admit"),
    };
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let durability = super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    let open = PhysicalRecordOpen::new(format, access, durability)
        .with_residency_policy(super::residency(format, super::RESIDENT_BYTES))
        .with_read_protection_policy(PhysicalReadProtectionPolicy::default())
        .with_physical_work_profile(profile);
    let serving = super::super::success(media.open_record_store(open));
    (serving, gate, activation)
}
