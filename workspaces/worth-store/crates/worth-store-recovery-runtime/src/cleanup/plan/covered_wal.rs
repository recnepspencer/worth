use worth_store::physical_runtime::recovery_wal::WalLsnRange;
use worth_store_recovery_physics::RetirementReleaseIntent;

use super::{RecoveryCleanupDeferralReason, RecoveryCleanupDispositionKind};
use crate::entry::PhysicalRecoveryLimitDeclaration;

/// What covered-WAL classification reads from one checkpoint-covered artifact.
#[derive(Clone, Copy)]
pub(super) struct CoveredWalFacts {
    pub(super) lsn_range: WalLsnRange,
    pub(super) byte_count: u64,
    pub(super) cleanup_safe: bool,
}

pub(super) struct CoveredWalBasis<'a> {
    /// Index of the first covered artifact the replay continuation needs.
    pub(super) protected_start: Option<usize>,
    pub(super) checkpoint_generation: Option<u64>,
    pub(super) release_intents: &'a [RetirementReleaseIntent],
    pub(super) unresolved_retirement: bool,
    pub(super) unresolved: bool,
    pub(super) limits: PhysicalRecoveryLimitDeclaration,
}

/// Classifies each covered artifact in WAL order. Everything from the first
/// retained index on stays, so pruning never opens a gap inside retained WAL.
pub(super) fn classify_covered_wal(
    covered: &[CoveredWalFacts],
    basis: CoveredWalBasis<'_>,
) -> Vec<RecoveryCleanupDispositionKind> {
    let retained_from = first_retained_covered_index(
        covered.iter().map(|facts| facts.lsn_range),
        basis.protected_start,
        basis.checkpoint_generation,
        basis.release_intents,
    );
    let mut candidate_count = 0_u64;
    let mut candidate_bytes = 0_u64;
    let mut kinds = Vec::with_capacity(covered.len());
    for (index, facts) in covered.iter().enumerate() {
        let kind = if retained_from.is_some_and(|start| index >= start) {
            RecoveryCleanupDispositionKind::Retained
        } else {
            checkpoint_covered_disposition(CheckpointCoveredWalDecision {
                cleanup_safe: facts.cleanup_safe,
                unresolved_retirement: basis.unresolved_retirement,
                unresolved: basis.unresolved,
                next_count: candidate_count + 1,
                next_bytes: candidate_bytes.checked_add(facts.byte_count),
                limits: basis.limits,
            })
        };
        if kind == RecoveryCleanupDispositionKind::Eligible {
            candidate_count += 1;
            candidate_bytes = candidate_bytes
                .checked_add(facts.byte_count)
                .expect("bounded cleanup byte sum");
        }
        kinds.push(kind);
    }
    kinds
}

/// The first covered index the plan retains: the replay continuation or the
/// first artifact still holding retirement authority, whichever comes first.
pub(super) fn first_retained_covered_index(
    covered: impl IntoIterator<Item = WalLsnRange>,
    protected_start: Option<usize>,
    checkpoint_generation: Option<u64>,
    intents: &[RetirementReleaseIntent],
) -> Option<usize> {
    let authority = checkpoint_generation.and_then(|generation| {
        covered
            .into_iter()
            .position(|range| holds_retirement_authority(range, generation, intents))
    });
    protected_start.into_iter().chain(authority).min()
}

/// Whether a covered artifact holds a retirement-release intent for a root past
/// the selected checkpoint. That intent is the only authority for the
/// checkpoint's retirement edge, so it outlives its resolved retirement until
/// a checkpoint at or past the candidate root is selected.
pub(super) fn holds_retirement_authority(
    artifact: WalLsnRange,
    checkpoint_generation: u64,
    intents: &[RetirementReleaseIntent],
) -> bool {
    intents.iter().any(|intent| {
        intent.candidate_generation() > checkpoint_generation
            && intent.lsn().start() < artifact.end_exclusive()
            && artifact.start() < intent.lsn().end_exclusive()
    })
}

pub(super) struct CheckpointCoveredWalDecision {
    pub(super) cleanup_safe: bool,
    pub(super) unresolved_retirement: bool,
    pub(super) unresolved: bool,
    pub(super) next_count: u64,
    pub(super) next_bytes: Option<u64>,
    pub(super) limits: PhysicalRecoveryLimitDeclaration,
}

pub(super) fn checkpoint_covered_disposition(
    decision: CheckpointCoveredWalDecision,
) -> RecoveryCleanupDispositionKind {
    if !decision.cleanup_safe {
        RecoveryCleanupDispositionKind::QuarantinedOrUnsupported
    } else if decision.unresolved_retirement {
        RecoveryCleanupDispositionKind::Deferred(
            RecoveryCleanupDeferralReason::UnresolvedRetirement,
        )
    } else if decision.unresolved {
        RecoveryCleanupDispositionKind::Deferred(
            RecoveryCleanupDeferralReason::UnresolvedOperationFate,
        )
    } else if decision.next_count > decision.limits.cleanup_candidates {
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::CandidateLimit)
    } else if decision
        .next_bytes
        .is_none_or(|bytes| bytes > decision.limits.cleanup_bytes)
    {
        RecoveryCleanupDispositionKind::Deferred(RecoveryCleanupDeferralReason::ByteLimit)
    } else {
        RecoveryCleanupDispositionKind::Eligible
    }
}
