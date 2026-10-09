use worth_store::physical_runtime::recovery_wal::{LogSequenceNumber, WalLsnRange};
use worth_store_recovery_physics::RetirementReleaseIntent;

use super::covered_wal::{
    checkpoint_covered_disposition, classify_covered_wal, holds_retirement_authority,
    CheckpointCoveredWalDecision, CoveredWalBasis, CoveredWalFacts,
};
use super::{RecoveryCleanupDeferralReason, RecoveryCleanupDispositionKind, RecoveryCleanupPlan};
use crate::entry::PhysicalRecoveryLimitDeclaration;

fn lsn(start: u64, end_exclusive: u64) -> WalLsnRange {
    WalLsnRange::new(
        LogSequenceNumber::new(start),
        LogSequenceNumber::new(end_exclusive),
    )
    .unwrap()
}

/// Four covered artifacts in WAL order; the third holds the intent at 15..16.
fn covered(cleanup_safe: [bool; 4]) -> [CoveredWalFacts; 4] {
    let ranges = [lsn(0, 5), lsn(5, 10), lsn(10, 17), lsn(17, 20)];
    std::array::from_fn(|index| CoveredWalFacts {
        lsn_range: ranges[index],
        byte_count: 1,
        cleanup_safe: cleanup_safe[index],
    })
}

fn classify(
    covered: &[CoveredWalFacts],
    protected_start: Option<usize>,
    checkpoint_generation: u64,
) -> Vec<RecoveryCleanupDispositionKind> {
    let intents = [RetirementReleaseIntent::new(lsn(15, 16), 14, 15, [7; 32])];
    classify_covered_wal(
        covered,
        CoveredWalBasis {
            protected_start,
            checkpoint_generation: Some(checkpoint_generation),
            release_intents: &intents,
            unresolved_retirement: false,
            unresolved: false,
            limits: cleanup_limits(10, 10),
        },
    )
}

/// A release writes its intent (15..16), a checkpoint at root 14 covers it,
/// root 15 publishes and the retirement resolves. Until a checkpoint at or
/// past root 15 is selected, the intent is the only authority for the
/// checkpoint's retirement edge, so the plan retains its covered WAL.
#[test]
fn resolved_release_intent_retains_covered_wal_until_a_checkpoint_passes_it() {
    use RecoveryCleanupDispositionKind::{Eligible, Retained};
    let safe = covered([true; 4]);
    assert_eq!(
        classify(&safe, None, 14),
        [Eligible, Eligible, Retained, Retained]
    );
    assert_eq!(classify(&safe, None, 15), [Eligible; 4]);
    assert_eq!(
        classify(&safe, Some(3), 14),
        [Eligible, Eligible, Retained, Retained]
    );
    assert_eq!(
        classify(&safe, Some(3), 15),
        [Eligible, Eligible, Eligible, Retained]
    );
    assert_eq!(
        classify(&safe, Some(1), 14),
        [Eligible, Retained, Retained, Retained]
    );
    // Retained WAL stays retained whatever its cleanup safety.
    let interrupted = covered([true, true, false, false]);
    assert_eq!(
        classify(&interrupted, None, 14),
        [Eligible, Eligible, Retained, Retained]
    );
}

#[test]
fn retirement_authority_needs_an_intent_past_the_checkpoint_inside_the_artifact() {
    let intents = [RetirementReleaseIntent::new(lsn(15, 16), 14, 15, [7; 32])];
    let artifact = lsn(10, 17);
    assert!(holds_retirement_authority(artifact, 14, &intents));
    assert!(!holds_retirement_authority(artifact, 15, &intents));
    assert!(!holds_retirement_authority(lsn(10, 15), 14, &intents));
    assert!(!holds_retirement_authority(lsn(16, 20), 14, &intents));
    assert!(holds_retirement_authority(lsn(15, 16), 14, &intents));
}

#[test]
fn interrupted_checkpoint_covered_wal_is_quarantined_before_limit_classification() {
    let kind = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: false,
        unresolved_retirement: false,
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
    let retirement = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved_retirement: true,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    let unresolved = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved_retirement: false,
        unresolved: true,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    let candidates = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved_retirement: false,
        unresolved: false,
        next_count: 2,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    let bytes = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved_retirement: false,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(2),
        limits: cleanup_limits(1, 1),
    });
    let eligible = checkpoint_covered_disposition(CheckpointCoveredWalDecision {
        cleanup_safe: true,
        unresolved_retirement: false,
        unresolved: false,
        next_count: 1,
        next_bytes: Some(1),
        limits: cleanup_limits(1, 1),
    });
    assert_eq!(
        retirement,
        RecoveryCleanupDispositionKind::Deferred(
            RecoveryCleanupDeferralReason::UnresolvedRetirement
        )
    );
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
