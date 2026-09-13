use super::{
    checkpoint_covered_disposition, CheckpointCoveredWalDecision, RecoveryCleanupDeferralReason,
    RecoveryCleanupDispositionKind, RecoveryCleanupPlan,
};
use crate::entry::PhysicalRecoveryLimitDeclaration;

#[test]
fn interrupted_checkpoint_covered_wal_is_quarantined_before_limit_classification() {
    let kind = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: false,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    assert_eq!(
        kind,
        RecoveryCleanupDispositionKind::QuarantinedOrUnsupported
    );
}

#[test]
fn cleanup_limit_dimensions_remain_causally_distinct() {
    let unresolved = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved: true,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    let candidates = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved: false,
        next_count: 2,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    let bytes = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(2),
        limits: cleanup_limits(1, 1),
    });
    let eligible = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    assert_eq!(
        unresolved,
        RecoveryCleanupDispositionKind::Deferred(
            RecoveryCleanupDeferralReason::UnresolvedOperationFate
        )
    );
    assert_eq!(
        candidates,
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::CandidateLimit)
    );
    assert_eq!(
        bytes,
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::ByteLimit)
    );
    assert_eq!(eligible, RecoveryCleanupDispositionKind::Eligible);
}

#[test]
fn store_execution_authority_does_not_replace_the_descriptive_plan_identity() {
    let mut plan = RecoveryCleanupPlan {
        identity: [0x11; 32],
        authority_identity: None,
        published_generation: 7,
        candidates: Vec::new(),
        dispositions: Vec::new(),
    };

    plan.bind_authority_identity([0x22; 32]);

    assert_eq!(plan.identity(), [0x11; 32]);
    assert_eq!(plan.authority_identity(), Some([0x22; 32]));
}

fn cleanup_limits(cleanup_candidates: u64, cleanup_bytes: u64) -> PhysicalRecoveryLimitDeclaration {
    PhysicalRecoveryLimitDeclaration {
        selector_candidates: 1,
        checkpoint_candidates: 1,
        manifest_bytes: 1,
        manifest_entries: 1,
        wal_segments: 1,
        wal_frames: 1,
        wal_bytes: 1,
        redo_targets: 1,
        redo_bytes: 1,
        distinct_pages_and_extents: 1,
        operation_bindings: 1,
        staging_bytes: 1,
        recovery_memory_bytes: 1,
        dirty_frames: 1,
        concurrent_commands: 1,
        publication_effects: 1,
        cleanup_candidates,
        cleanup_bytes,
        observation_bytes: 1,
    }
}
