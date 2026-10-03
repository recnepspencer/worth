//! Release custody across planning and published-root rebind. Tier custody is
//! orthogonal and remains owned by its independent phase contract.

use worth_store_recovery_physics::{
    VerifiedEffectiveReleaseHeadRosterV14, VerifiedOrderedHistoricalReleaseCustody,
    VerifiedPendingWalReleaseCustody, VerifiedSelectedNoReleaseCustody,
    VerifiedSelectedReleaseHeadCustodyV2, VerifiedSelectedReleaseHeadReplayV14,
};

/// Planning may hold pre-publication pending storage, but cannot present it as
/// a final effective roster before the exact published root is admitted.
pub(crate) enum PlanningCustody {
    Unresolved,
    NoCheckpoint,
    NoRelease(VerifiedSelectedNoReleaseCustody),
    SourceHeads(VerifiedSelectedReleaseHeadCustodyV2),
    PendingPrepared {
        claim: VerifiedPendingWalReleaseCustody,
        replay: PendingReleaseReplay,
    },
    OrderedCompleted {
        claim: VerifiedOrderedHistoricalReleaseCustody,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
}

/// Head and optional directory evidence belong to the same admitted WAL member.
/// A directory effect cannot be carried independently of its release replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PendingReleaseReplay {
    head: VerifiedSelectedReleaseHeadReplayV14,
    directory: Option<worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement>,
}

impl PendingReleaseReplay {
    pub(crate) fn new(
        projection: &worth_store_recovery_physics::PhysicalRedoProjection,
        head: VerifiedSelectedReleaseHeadReplayV14,
        directory: Option<worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement>,
    ) -> Option<Self> {
        let worth_store_physical_format::PersistedPhysicalRecoveryOperation::RecordsDropped {
            head_effect: Some(effect),
            directory_replacement,
            ..
        } = projection.materialization().operation()
        else {
            return None;
        };
        if head.operation() != projection.operation()
            || head.effect() != effect
            || directory_replacement.is_some() != directory.is_some()
            || directory.as_ref().is_some_and(|proof| {
                proof.operation() != head.operation()
                    || proof.candidate_generation() != effect.result_root().generation()
                    || directory_replacement.as_ref().is_none_or(|binding| {
                        proof.source_binding() != binding.expected_previous()
                            || proof.result_binding().directory_record()
                                != binding.next().record().record()
                    })
            })
        {
            return None;
        }
        Some(Self { head, directory })
    }

    pub(crate) fn head(&self) -> &VerifiedSelectedReleaseHeadReplayV14 {
        &self.head
    }
    pub(crate) fn directory(
        &self,
    ) -> Option<&worth_store_recovery_physics::VerifiedReleasedDirectoryReplacement> {
        self.directory.as_ref()
    }
    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        self.head.owned_heap_bytes()
    }
    pub(crate) fn into_head(self) -> VerifiedSelectedReleaseHeadReplayV14 {
        self.head
    }
}

/// Only complete custody states cross into Store's post-reopen construction.
pub(crate) enum CustodyState {
    NoCheckpoint,
    NoRelease(VerifiedSelectedNoReleaseCustody),
    SourceHeads(VerifiedSelectedReleaseHeadCustodyV2),
    Pending {
        claim: VerifiedPendingWalReleaseCustody,
        replay: VerifiedSelectedReleaseHeadReplayV14,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
    OrderedCompleted {
        claim: VerifiedOrderedHistoricalReleaseCustody,
        effective_heads: VerifiedEffectiveReleaseHeadRosterV14,
    },
}
