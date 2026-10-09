use std::{
    num::{NonZeroU32, NonZeroU64},
    path::Path,
};

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    FilesystemMediaAdmission, PhysicalEffectRecoveryObligation,
    PhysicalMutationIdempotencyMaterial, PhysicalRecordOpen, PhysicalRuntimeAdmission,
    PhysicalStore, PhysicalWalGroupBarrierOutcome, PhysicalWorkOperationFamily,
    PhysicalWorkRecoveryDisposition, PhysicalWorkRecoveryTarget, ServingPhysicalRuntime,
};
use worth_store_physical_backend::{
    CertificationMediaFaultActivation, FilesystemAccessPosture, MediaCounterSnapshot,
    MediaOperationRole,
};

use super::super::{configuration, durability_with_group_limit};
use super::wal_barrier::append;

#[test]
fn wal_append_retires_its_record_durably_and_its_barrier_journals_nothing() {
    let parent = tempfile::tempdir().unwrap();
    let serving = super::super::serving_from_initialization(&parent.path().join("store"));
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    // The first commit also creates the WAL segment; measure a steady one.
    assert!(matches!(
        commit(&serving, 93),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let started = serving.media_counters();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([94; 32]),
    );

    let before = serving.media_counters();
    assert_eq!(
        (
            attempts(before, MediaOperationRole::CreateNew)
                - attempts(started, MediaOperationRole::CreateNew),
            attempts(before, MediaOperationRole::Delete)
                - attempts(started, MediaOperationRole::Delete),
            before.directory_syncs() - started.directory_syncs(),
        ),
        (1, 1, 2),
        "the append journals one record and retires it durably"
    );
    assert!(matches!(
        submission.synchronize_appended_wal_group(appended),
        PhysicalWalGroupBarrierOutcome::Durable(_)
    ));
    let acknowledged = serving.media_counters();
    let delta = |role| attempts(acknowledged, role) - attempts(before, role);
    assert_eq!(
        delta(MediaOperationRole::CreateNew),
        0,
        "a WAL barrier writes no record"
    );
    assert_eq!(
        delta(MediaOperationRole::Delete),
        0,
        "a WAL barrier retires no record"
    );
    assert_eq!(
        acknowledged.directory_syncs() - before.directory_syncs(),
        0,
        "a WAL barrier synchronizes no journal directory"
    );
    serving.close();
}

#[test]
fn failed_wal_flush_writes_its_record_and_reopen_fences_it() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    super::super::serving_from_initialization(&store_root).close();
    let (serving, activations) = serving_with_faults(
        &store_root,
        &[(
            MediaOperationRole::SynchronizeFileState,
            MediaFaultDirective::IndeterminateAfterEffect,
        )],
    );
    assert_eq!(
        indeterminate_recovery(barrier_after_arming(&serving, &activations, 95)),
        PhysicalEffectRecoveryObligation::Retained
    );
    assert!(serving.close_plan().execute().requires_inspection());

    let reopened = super::super::serving_from_open(&store_root);
    let obligations = reopened.physical_recovery_obligations();
    assert_eq!(obligations.len(), 1);
    assert_eq!(
        obligations[0].family(),
        PhysicalWorkOperationFamily::DurabilityBarrier
    );
    assert!(matches!(
        obligations[0].target(),
        PhysicalWorkRecoveryTarget::WalArtifactInterval { .. }
    ));
    assert_eq!(
        obligations[0].recovery_disposition(),
        PhysicalWorkRecoveryDisposition::InspectionRequired
    );
    assert!(reopened.close_plan().execute().requires_inspection());
}

#[test]
fn failed_wal_flush_whose_record_write_also_fails_reports_both_failures() {
    let parent = tempfile::tempdir().unwrap();
    let store_root = parent.path().join("store");
    super::super::serving_from_initialization(&store_root).close();
    let (serving, activations) = serving_with_faults(
        &store_root,
        &[
            (
                MediaOperationRole::SynchronizeFileState,
                MediaFaultDirective::IndeterminateAfterEffect,
            ),
            (
                MediaOperationRole::CreateNew,
                MediaFaultDirective::FailBefore {
                    kind: std::io::ErrorKind::Other,
                    raw_os_error: None,
                },
            ),
        ],
    );
    // The barrier's typed failure carries both: its outcome is indeterminate
    // and its recovery record is missing.
    assert_eq!(
        indeterminate_recovery(barrier_after_arming(&serving, &activations, 96)),
        PhysicalEffectRecoveryObligation::RetainedWithoutRecord
    );
    assert!(activations
        .iter()
        .all(CertificationMediaFaultActivation::is_consumed));

    assert!(serving.close_plan().execute().requires_inspection());
    // Only the failed runtime fenced the flush: no record reaches reopen.
    let reopened = super::super::serving_from_open(&store_root);
    assert!(reopened.physical_recovery_obligations().is_empty());
    reopened.close();
}

fn commit(serving: &ServingPhysicalRuntime, material: u8) -> PhysicalWalGroupBarrierOutcome {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([material; 32]),
    );
    submission.synchronize_appended_wal_group(appended)
}

fn indeterminate_recovery(
    outcome: PhysicalWalGroupBarrierOutcome,
) -> PhysicalEffectRecoveryObligation {
    match outcome {
        PhysicalWalGroupBarrierOutcome::Indeterminate(barrier) => barrier.recovery_obligation(),
        _ => panic!("a failed WAL flush must leave the barrier indeterminate"),
    }
}

fn barrier_after_arming(
    serving: &ServingPhysicalRuntime,
    activations: &[CertificationMediaFaultActivation],
    material: u8,
) -> PhysicalWalGroupBarrierOutcome {
    let (_, placement, _) = configuration();
    let submission = serving.certification_record_submission();
    let appended = append(
        &submission,
        placement,
        PhysicalMutationIdempotencyMaterial::new([material; 32]),
    );
    for activation in activations {
        activation.arm().unwrap();
    }
    submission.synchronize_appended_wal_group(appended)
}

fn serving_with_faults(
    root: &Path,
    faults: &[(MediaOperationRole, MediaFaultDirective)],
) -> (
    ServingPhysicalRuntime,
    Vec<CertificationMediaFaultActivation>,
) {
    let admission =
        FilesystemMediaAdmission::certification(FilesystemAccessPosture::CoordinatedServiceAccount);
    let authority = admission.fault_schedule_authority();
    let activations = faults
        .iter()
        .map(|_| authority.one_shot_activation())
        .collect::<Vec<_>>();
    let rules = faults
        .iter()
        .zip(&activations)
        .map(|((role, directive), activation)| {
            authority
                .rule(*role, 1, directive.clone())
                .for_nth_identified_operation_after_activation(activation.clone(), NonZeroU64::MIN)
        })
        .collect();
    let schedule = authority.schedule(rules).unwrap();
    let runtime = PhysicalStore::admit(PhysicalRuntimeAdmission::new(root).unwrap()).unwrap();
    let media = match runtime
        .try_admit_filesystem_media(admission.with_fault_schedule(schedule))
        .into_raw()
    {
        TransitionOutcome::Success(media) => media,
        _ => panic!("fault-scheduled media admission must succeed"),
    };
    let policy = durability_with_group_limit(&media, NonZeroU32::new(32).unwrap());
    let (format, _, access) = configuration();
    let serving = super::super::success(
        media.open_record_store(PhysicalRecordOpen::new(format, access, policy)),
    );
    (serving, activations)
}

fn attempts(counters: MediaCounterSnapshot, role: MediaOperationRole) -> u64 {
    counters.attempts_for(role)
}
