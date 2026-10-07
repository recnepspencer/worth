use worth_proof::TransitionOutcome;
use worth_signal::facade::TemporalDuration;
use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    AdmittedRecordPlacementPolicy, FilesystemMediaAdmission, PhysicalMutationAdmissionDisposition,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyKey, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalRecordInitialization,
    PhysicalRecordOpen, PhysicalRuntimeAdmission, PhysicalStore, PhysicalWalGroupBarrierOutcome,
    PhysicalWalReclamationObservation, PreparedPhysicalMutation, RecordAppendBatch,
    ServingPhysicalRuntime,
};
use worth_store_physical_backend::{
    CertificationMediaFaultActivation, FilesystemAccessPosture, MediaOperationRole, MediaPauseGate,
};

use super::super::{configuration, durability_with_wal_policy, success};
use super::checkpoint_retained_wal_tail::{
    durable_group, publish_after_foreground_rotations, rotation_world,
};
use super::independent_wal_oracle::inspect_wal_inventory;
use super::wal_rotation::{append_group, wal_policy};

#[test]
fn reclaimed_wal_prefix_remains_absent_after_fresh_process_reopen() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement) = rotation_world(&store_root);
    let submission = serving.certification_record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([51; 32]))
        .unwrap();
    let base = prepare_with_key(&submission, placement, key.clone(), b"reopen-base");
    let base_identity = base.mutation_identity();
    let base = append_group(&submission, vec![base]);
    assert!(matches!(
        submission.synchronize_appended_wal_group(base),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let (completed, _) = publish_after_foreground_rotations(&serving, submission, placement);
    assert_reclaimed_prefix(&completed.wal_reclamation(), 1);
    assert_unmaterialized_candidates(&store_root, &serving);
    assert_eq!(
        serving
            .certification_record_submission()
            .wal_observation()
            .unwrap()
            .appended_frames(),
        5
    );
    let retained_charge = serving.certification_charged_growth_bytes();
    serving.close();

    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1)]
    );
    let media_owner = super::super::media(&store_root);
    let durability = durability_with_wal_policy(&media_owner, wal_policy(6));
    let (format, placement, access) = configuration();
    let reopened =
        success(media_owner.open_record_store(PhysicalRecordOpen::new(format, access, durability)));
    let observation = reopened
        .certification_record_submission()
        .wal_observation()
        .unwrap();
    assert_eq!(observation.active_segment_count(), 2);
    assert_eq!(observation.reopened_frames(), 4);
    assert!(!observation.sealed_for_inspection());
    assert_unmaterialized_candidates(&store_root, &reopened);
    // These five single-page mutations reached WAL durability, not data
    // dispatch. Reopen restores retained media, not their transient candidate
    // reservations. Completed-publication parity is a separate journey.
    let pending_candidate_bytes = 5 * u64::from(format.declaration().page_size().bytes());
    assert_eq!(
        retained_charge,
        reopened.certification_charged_growth_bytes() + pending_candidate_bytes,
        "reopen must not resurrect unwritten candidate reservations or reclaimed WAL"
    );
    let replay = prepare_with_key(
        &reopened.certification_record_submission(),
        placement,
        key,
        b"reopen-base",
    );
    assert_eq!(
        replay.disposition(),
        PhysicalMutationAdmissionDisposition::DuplicateUnresolved
    );
    assert_eq!(replay.mutation_identity(), base_identity);
    reopened.close();
}

fn assert_unmaterialized_candidates(root: &std::path::Path, serving: &ServingPhysicalRuntime) {
    assert!(std::fs::read_dir(root.join("families/records/segments"))
        .unwrap()
        .next()
        .is_none());
    assert_eq!(
        serving
            .observer()
            .acquisition_snapshot()
            .unwrap()
            .root_generation(),
        1
    );
}

fn prepare_with_key(
    submission: &worth_store::physical_runtime::PhysicalRecordSubmission,
    placement: AdmittedRecordPlacementPolicy,
    key: PhysicalMutationIdempotencyKey,
    payload: &[u8],
) -> PreparedPhysicalMutation {
    let request = PhysicalMutationRequest::platform_durable(
        key,
        PhysicalMutationDeadline::at(TemporalDuration::temporal_duration(1_000).unwrap()),
    );
    match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter([payload]).unwrap(),
            placement,
            request,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("exact-key reclamation replay was not admitted"),
    }
}

#[test]
fn denied_reclamation_preserves_the_exact_live_inventory_for_retry() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement, activation) = fault_rotation_world(
        &store_root,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(61, b"denied-base")]);
    activation.arm().unwrap();
    let (completed, _) =
        publish_after_foreground_rotations(&serving, submission.clone(), placement);

    match completed.wal_reclamation() {
        PhysicalWalReclamationObservation::DeferredBeforeEffect(report) => {
            assert_eq!(report.planned_segments(), 1);
            assert_eq!(report.reclaimed_segments(), 0);
            assert_eq!(report.reclaimed_bytes(), 0);
            assert!(report.first_unreclaimed().is_some());
        }
        other => panic!("denied reclamation was not reported as preeffect: {other:?}"),
    }
    assert!(activation.is_consumed());
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(1, 1), (2, 1), (3, 1)]
    );
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 0);
    assert!(!wal.sealed_for_inspection());
    serving.close();
}

#[test]
fn second_delete_denial_preserves_the_exact_partially_reclaimed_prefix() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let world = second_delete_fault_rotation_world(
        &store_root,
        MediaFaultDirective::FailBefore {
            kind: std::io::ErrorKind::Other,
            raw_os_error: None,
        },
    );
    let submission = world.serving.certification_record_submission();
    durable_group(&submission, world.placement, &[(81, b"partial-base")]);
    durable_group(&submission, world.placement, &[(82, b"partial-second")]);
    world.pause_activation.arm().unwrap();
    let (completed, _) = std::thread::scope(|scope| {
        let checkpoint = scope.spawn(|| {
            publish_after_foreground_rotations(&world.serving, submission.clone(), world.placement)
        });
        world.first_delete_gate.wait_until_reached();
        world.fail_activation.arm().unwrap();
        world.first_delete_gate.release();
        checkpoint.join().unwrap()
    });

    match completed.wal_reclamation() {
        PhysicalWalReclamationObservation::DeferredBeforeEffect(report) => {
            assert_eq!(report.planned_segments(), 2);
            assert_eq!(report.reclaimed_segments(), 1);
            assert!(report.reclaimed_bytes() > 0);
            let first_unreclaimed = report.first_unreclaimed().unwrap();
            assert_eq!(first_unreclaimed.segment().get(), 2);
            assert_eq!(first_unreclaimed.generation().get(), 1);
        }
        other => panic!("partial reclamation did not preserve retry truth: {other:?}"),
    }
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1), (4, 1)]
    );
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 1);
    assert!(!wal.sealed_for_inspection());
    assert!(world.pause_activation.is_consumed());
    assert!(world.fail_activation.is_consumed());
    world.serving.close();
}

#[test]
fn indeterminate_reclamation_seals_without_consuming_inventory_truth() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    let (serving, placement, activation) =
        fault_rotation_world(&store_root, MediaFaultDirective::IndeterminateAfterEffect);
    let submission = serving.certification_record_submission();
    durable_group(&submission, placement, &[(71, b"indeterminate-base")]);
    activation.arm().unwrap();
    let (completed, _) =
        publish_after_foreground_rotations(&serving, submission.clone(), placement);

    match completed.wal_reclamation() {
        PhysicalWalReclamationObservation::InspectionRequired(report) => {
            assert_eq!(report.planned_segments(), 1);
            assert_eq!(report.reclaimed_segments(), 0);
            assert!(report.first_unreclaimed().is_some());
        }
        other => panic!("indeterminate reclamation did not require inspection: {other:?}"),
    }
    assert!(activation.is_consumed());
    let wal = submission.wal_observation().unwrap();
    assert_eq!(wal.active_segment_count(), 3);
    assert_eq!(wal.reclaimed_segments(), 0);
    assert!(wal.sealed_for_inspection());
    assert_eq!(
        inspect_wal_inventory(&store_root).unwrap().segments(),
        &[(2, 1), (3, 1)]
    );
    serving.close();
}

fn fault_rotation_world(
    store_root: &std::path::Path,
    directive: MediaFaultDirective,
) -> (
    ServingPhysicalRuntime,
    AdmittedRecordPlacementPolicy,
    CertificationMediaFaultActivation,
) {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![authority
            .rule(MediaOperationRole::Delete, 1, directive)
            .for_next_identified_operation_after_activation(
                activation.clone(),
            )])
        .unwrap();
    let (serving, placement) =
        admit_fault_rotation_world(store_root, admission.with_fault_schedule(schedule));
    (serving, placement, activation)
}

struct SecondDeleteFaultRotationWorld {
    serving: ServingPhysicalRuntime,
    placement: AdmittedRecordPlacementPolicy,
    first_delete_gate: MediaPauseGate,
    pause_activation: CertificationMediaFaultActivation,
    fail_activation: CertificationMediaFaultActivation,
}

fn second_delete_fault_rotation_world(
    store_root: &std::path::Path,
    directive: MediaFaultDirective,
) -> SecondDeleteFaultRotationWorld {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let first_delete_gate = authority.pause_gate();
    let pause_activation = authority.one_shot_activation();
    let fail_activation = authority.one_shot_activation();
    let schedule = authority
        .schedule(vec![
            authority
                .rule(MediaOperationRole::Delete, 1, directive)
                .for_next_identified_operation_after_activation(fail_activation.clone()),
            authority
                .rule(
                    MediaOperationRole::Delete,
                    1,
                    MediaFaultDirective::PauseBefore(first_delete_gate.clone()),
                )
                .for_next_identified_operation_after_activation(pause_activation.clone()),
        ])
        .unwrap();
    let (serving, placement) =
        admit_fault_rotation_world(store_root, admission.with_fault_schedule(schedule));
    SecondDeleteFaultRotationWorld {
        serving,
        placement,
        first_delete_gate,
        pause_activation,
        fail_activation,
    }
}

fn admit_fault_rotation_world(
    store_root: &std::path::Path,
    admission: FilesystemMediaAdmission,
) -> (ServingPhysicalRuntime, AdmittedRecordPlacementPolicy) {
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(store_root).unwrap()).unwrap();
    let media_owner = match runtime.try_admit_filesystem_media(admission).into_raw() {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled WAL reclamation media was not admitted"),
    };
    let durability = durability_with_wal_policy(&media_owner, wal_policy(6));
    let (format, placement, access) = configuration();
    let serving = success(
        media_owner.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
    );
    (serving, placement)
}

fn assert_reclaimed_prefix(
    observation: &PhysicalWalReclamationObservation,
    expected_segments: u32,
) {
    match observation {
        PhysicalWalReclamationObservation::Reclaimed(report) => {
            assert_eq!(report.planned_segments(), expected_segments);
            assert_eq!(report.reclaimed_segments(), expected_segments);
            assert!(report.reclaimed_bytes() > 0);
            assert_eq!(report.first_unreclaimed(), None);
        }
        other => panic!("checkpoint did not reclaim its obsolete WAL prefix: {other:?}"),
    }
}
