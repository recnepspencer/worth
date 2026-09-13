use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;
use worth_store_recovery_runtime::{
    RecoveryCleanupDeferralReason, RecoveryCleanupDisposition, RecoveryCleanupDispositionKind,
    RecoveryCleanupPosture,
};

use super::{required_text, EXPECTED_POSTURE};

pub(super) fn assert_expected_posture(
    posture: &RecoveryCleanupPosture,
    disposition: &RecoveryCleanupDisposition,
    expected_identity: WalSegmentArtifactIdentity,
    expected_deferred: u64,
    expected_revalidation_bytes: u64,
) {
    let evidence = posture.evidence();
    match required_text(EXPECTED_POSTURE).as_str() {
        "complete" => assert_complete(
            posture,
            disposition,
            expected_identity,
            expected_revalidation_bytes,
        ),
        "byte-limit" => assert_deferred(
            posture,
            disposition,
            RecoveryCleanupDeferralReason::ByteLimit,
        ),
        "unresolved" => assert_deferred(
            posture,
            disposition,
            RecoveryCleanupDeferralReason::UnresolvedOperationFate,
        ),
        "candidate-limit" => {
            assert!(matches!(posture, RecoveryCleanupPosture::Deferred(_)));
            assert_eq!(
                disposition.kind(),
                RecoveryCleanupDispositionKind::SafelyRemoved
            );
            assert_eq!(evidence.counters().actions_planned, 1);
            assert_eq!(evidence.counters().actions_completed, 1);
            assert_eq!(evidence.counters().terminal_binding_evaluations, 1);
            assert_eq!(evidence.counters().live_media_handles_after_close, 0);
            assert_eq!(evidence.counters().actions_deferred, expected_deferred);
            assert!(evidence
                .dispositions()
                .iter()
                .any(|disposition| disposition.kind()
                    == RecoveryCleanupDispositionKind::Deferred(
                        RecoveryCleanupDeferralReason::CandidateLimit,
                    )));
        }
        "cancel-0" => assert_cancelled(posture, disposition, 0),
        "cancel-1" => assert_cancelled(posture, disposition, 1),
        expected => panic!("unsupported expected cleanup posture: {expected}"),
    }
}

fn assert_cancelled(
    posture: &RecoveryCleanupPosture,
    disposition: &RecoveryCleanupDisposition,
    settled_actions: u64,
) {
    let evidence = posture.evidence();
    assert!(matches!(posture, RecoveryCleanupPosture::Deferred(_)));
    let expected_kind = if settled_actions == 0 {
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::Cancelled)
    } else {
        RecoveryCleanupDispositionKind::SafelyRemoved
    };
    assert_eq!(disposition.kind(), expected_kind);
    assert_eq!(evidence.counters().cancellation_requests, 1);
    assert_eq!(evidence.counters().terminal_binding_evaluations, 1);
    assert_eq!(evidence.counters().actions_completed, settled_actions);
    assert!(evidence.counters().actions_cancelled > 0);
    assert!(evidence.counters().bytes_cancelled > 0);
    assert!(matches!(
        evidence.deferrals().last(),
        Some(worth_store_recovery_runtime::RecoveryCleanupDeferralEvidence::Cancelled {
            settled_actions: actual,
            ..
        }) if *actual == settled_actions
    ));
}

fn assert_complete(
    posture: &RecoveryCleanupPosture,
    disposition: &RecoveryCleanupDisposition,
    expected_identity: WalSegmentArtifactIdentity,
    expected_revalidation_bytes: u64,
) {
    let evidence = posture.evidence();
    assert!(matches!(posture, RecoveryCleanupPosture::Complete(_)));
    assert_eq!(
        disposition.kind(),
        RecoveryCleanupDispositionKind::SafelyRemoved
    );
    assert_eq!(evidence.performed_removals().len(), 1);
    assert_eq!(evidence.counters().terminal_binding_evaluations, 1);
    let occurrence = evidence.performed_removals()[0].occurrence();
    assert_ne!(occurrence.plan(), [0; 32]);
    assert_ne!(evidence.plan_identity(), [0; 32]);
    assert_ne!(
        occurrence.plan(),
        evidence.plan_identity(),
        "the Store execution authority must not replace the descriptive cleanup-plan identity",
    );
    assert_eq!(occurrence.artifact().segment(), expected_identity.segment());
    assert_eq!(
        occurrence.artifact().generation(),
        expected_identity.generation()
    );
    assert_eq!(evidence.counters().artifact_revalidation_reads_attempted, 2);
    assert_eq!(evidence.counters().artifact_revalidation_reads_completed, 2);
    assert_eq!(evidence.counters().artifact_revalidation_read_failures, 0);
    assert_eq!(evidence.counters().artifact_revalidation_mismatches, 0);
    assert_eq!(
        evidence.counters().artifact_revalidation_bytes_read,
        expected_revalidation_bytes
    );
}

fn assert_deferred(
    posture: &RecoveryCleanupPosture,
    disposition: &RecoveryCleanupDisposition,
    reason: RecoveryCleanupDeferralReason,
) {
    assert!(matches!(posture, RecoveryCleanupPosture::Deferred(_)));
    assert_eq!(
        disposition.kind(),
        RecoveryCleanupDispositionKind::Deferred(reason)
    );
    assert!(posture.evidence().performed_removals().is_empty());
    assert_eq!(posture.evidence().counters().freshness_evaluations, 0);
    assert_eq!(
        posture.evidence().counters().terminal_binding_evaluations,
        0
    );
}
