use worth_store::physical_runtime::certification::MediaFaultDirective;
use worth_store::physical_runtime::{
    PhysicalRecoveryCleanupRemovalDenialKind, RecoveryCleanupArtifactRevalidationDenial,
    RecoveryCleanupRemovalDenialCause,
};
use worth_store_recovery_runtime::{
    PhysicalRecoveryOutcome, RecoveryCleanupDeferralEvidence, RecoveryCleanupPosture,
};

use super::world::{
    cleanup_fault, cleanup_world, empty_fault_schedule, recover_with_schedule, reopen_with_schedule,
};

#[path = "revalidation/checkpoint_replacement.rs"]
mod checkpoint_replacement;
#[path = "revalidation/denial_matrix.rs"]
mod denial_matrix;

#[test]
fn cleanup_denial_retains_the_artifact_and_exact_backend_failure() {
    let world = cleanup_world("cleanup-denied");
    let candidate = world.oldest_wal();
    let candidate_bytes = std::fs::metadata(&candidate).unwrap().len();
    let checkpoint_bytes = current_checkpoint_bytes(&world.root);
    let schedule = cleanup_fault(MediaFaultDirective::FailBefore {
        kind: std::io::ErrorKind::PermissionDenied,
        raw_os_error: None,
    });
    let handoff = recover_with_schedule(&world.root, schedule);
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("cleanup denial produces deferred recovered handoff")
    };
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
    assert_eq!(evidence.counters().denied_before_effect, 1);
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact denied cleanup settlement")
    };
    let physical = denial.physical().expect("backend denial is retained");
    assert_eq!(
        physical.failure().kind(),
        worth_store::physical_runtime::ArtifactTreeFailureKind::DeniedBeforeEffect
    );
    assert_eq!(
        physical.failure().io_kind(),
        Some(std::io::ErrorKind::PermissionDenied)
    );
    assert!(matches!(
        physical.cause(),
        RecoveryCleanupRemovalDenialCause::Removal(_)
    ));
    assert_eq!(physical.revalidation().reads_attempted(), 2);
    assert_eq!(physical.revalidation().reads_completed(), 2);
    assert_eq!(
        physical.revalidation().bytes_read(),
        checkpoint_bytes + candidate_bytes
    );
    assert_eq!(evidence.counters().artifact_revalidation_reads_attempted, 2);
    assert_eq!(evidence.counters().artifact_revalidation_reads_completed, 2);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        checkpoint_bytes + candidate_bytes
    );
}

#[test]
fn cleanup_revalidates_exact_wal_bytes_before_deletion() {
    let world = cleanup_world("cleanup-wal-substitution");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    let mut substituted = std::fs::read(&candidate).unwrap();
    let candidate_bytes = substituted.len() as u64;
    let checkpoint_bytes = current_checkpoint_bytes(&world.root);
    *substituted.last_mut().expect("candidate WAL is nonempty") ^= 0xff;
    std::fs::write(&candidate, substituted).unwrap();

    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("stale WAL bytes remain deferred cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("substituted WAL bytes must deny cleanup")
    };
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact digest mismatch")
    };
    assert!(matches!(
        denial.kind(),
        PhysicalRecoveryCleanupRemovalDenialKind::Media(
            RecoveryCleanupRemovalDenialCause::Revalidation(
                RecoveryCleanupArtifactRevalidationDenial::DigestMismatch { .. }
            )
        )
    ));
    let progress = denial.physical().unwrap().revalidation();
    assert_eq!(progress.reads_attempted(), 2);
    assert_eq!(progress.reads_completed(), 2);
    assert_eq!(progress.bytes_read(), checkpoint_bytes + candidate_bytes);
    assert_eq!(evidence.counters().artifact_revalidation_mismatches, 1);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        checkpoint_bytes + candidate_bytes
    );
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
}

#[test]
fn cleanup_revalidation_read_failure_retains_exact_zero_byte_progress() {
    let world = cleanup_world("cleanup-wal-read-failure");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    let checkpoint_bytes = current_checkpoint_bytes(&world.root);
    std::fs::remove_file(candidate).unwrap();

    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("reread failure remains deferred cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("missing cleanup artifact must defer")
    };
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact reread failure")
    };
    assert!(matches!(
        denial.kind(),
        PhysicalRecoveryCleanupRemovalDenialKind::Media(
            RecoveryCleanupRemovalDenialCause::Revalidation(
                RecoveryCleanupArtifactRevalidationDenial::Read(_)
            )
        )
    ));
    let progress = denial.physical().unwrap().revalidation();
    assert_eq!(progress.reads_attempted(), 2);
    assert_eq!(progress.reads_completed(), 1);
    assert_eq!(progress.bytes_read(), checkpoint_bytes);
    assert_eq!(evidence.counters().artifact_revalidation_reads_attempted, 2);
    assert_eq!(evidence.counters().artifact_revalidation_reads_completed, 1);
    assert_eq!(evidence.counters().artifact_revalidation_read_failures, 1);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        checkpoint_bytes
    );
}

#[test]
fn missing_wal_directory_retains_checkpoint_read_and_scheduled_denial_evidence() {
    let world = cleanup_world("cleanup-wal-directory-read-failure");
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    let checkpoint_bytes = current_checkpoint_bytes(&world.root);
    let wal_directory = world.root.join("families").join("wal");
    std::fs::rename(&wal_directory, world.root.join("families").join("wal-away")).unwrap();

    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("missing WAL directory remains deferred cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("missing WAL directory must defer cleanup")
    };
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact WAL directory read failure")
    };
    assert!(matches!(
        denial.kind(),
        PhysicalRecoveryCleanupRemovalDenialKind::Media(
            RecoveryCleanupRemovalDenialCause::Revalidation(
                RecoveryCleanupArtifactRevalidationDenial::Read(_)
            )
        )
    ));
    let physical = denial.physical().expect("physical denial is retained");
    assert!(physical.queue().is_some());
    assert_eq!(physical.revalidation().reads_attempted(), 2);
    assert_eq!(physical.revalidation().reads_completed(), 1);
    assert_eq!(physical.revalidation().bytes_read(), checkpoint_bytes);
    assert_eq!(evidence.counters().artifact_revalidation_reads_attempted, 2);
    assert_eq!(evidence.counters().artifact_revalidation_reads_completed, 1);
    assert_eq!(evidence.counters().artifact_revalidation_read_failures, 1);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        checkpoint_bytes
    );
    assert!(evidence.performed_removals().is_empty());
}

#[test]
fn cleanup_rejects_a_distinct_valid_persisted_checkpoint_before_wal_revalidation() {
    let world = cleanup_world("cleanup-checkpoint-substitution");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    let checkpoint_bytes = checkpoint_replacement::replace_with_distinct_valid_checkpoint(
        &world.root,
        reopened.store_identity(),
    );

    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("checkpoint substitution remains deferred cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("a different persisted checkpoint must deny cleanup")
    };
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact checkpoint mismatch")
    };
    assert!(matches!(
        denial.kind(),
        PhysicalRecoveryCleanupRemovalDenialKind::Media(
            RecoveryCleanupRemovalDenialCause::Revalidation(
                RecoveryCleanupArtifactRevalidationDenial::CheckpointDigestMismatch { .. }
            )
        )
    ));
    let progress = denial.physical().unwrap().revalidation();
    assert_eq!(progress.reads_attempted(), 1);
    assert_eq!(progress.reads_completed(), 1);
    assert_eq!(progress.bytes_read(), checkpoint_bytes);
    assert_eq!(evidence.counters().artifact_revalidation_mismatches, 1);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        checkpoint_bytes
    );
    assert!(candidate.exists());
    assert!(evidence.performed_removals().is_empty());
}

#[test]
fn cleanup_binding_substitution_denies_in_store_before_backend_revalidation_or_effect() {
    let world = cleanup_world("cleanup-authorization-substitution");
    let candidate = world.oldest_wal();
    let reopened = reopen_with_schedule(&world.root, empty_fault_schedule());
    reopened.certification_substitute_cleanup_authorization();

    let PhysicalRecoveryOutcome::Recovered(handoff) = reopened.finish() else {
        panic!("authorization mismatch remains deferred cleanup debt")
    };
    let RecoveryCleanupPosture::Deferred(evidence) = handoff.cleanup_posture() else {
        panic!("authorization mismatch must defer cleanup")
    };
    let [RecoveryCleanupDeferralEvidence::DeniedBeforeEffect { denial, .. }] = evidence.deferrals()
    else {
        panic!("one exact authorization denial")
    };
    assert!(matches!(
        denial.kind(),
        PhysicalRecoveryCleanupRemovalDenialKind::InvalidCommand
    ));
    assert!(denial.physical().is_none());
    assert!(denial.scheduler().is_none());
    assert!(denial.signal().is_some());
    assert_eq!(evidence.counters().actions_attempted, 1);
    assert_eq!(evidence.counters().removal_scheduler_submitted, 1);
    assert_eq!(evidence.counters().removal_scheduler_settled, 0);
    assert_eq!(evidence.counters().scheduler_commands_submitted, 2);
    assert_eq!(evidence.counters().scheduler_commands_settled, 1);
    assert_eq!(evidence.counters().artifact_revalidation_reads_attempted, 0);
    assert_eq!(evidence.counters().performed_effects, 0);
    assert!(evidence.performed_removals().is_empty());
    assert!(candidate.exists());
}

fn current_checkpoint_bytes(root: &std::path::Path) -> u64 {
    std::fs::metadata(root.join("families").join("checkpoint.current"))
        .unwrap()
        .len()
}
