use crate::{ChunkTreeRoot, LogicalContentDigest};

use super::super::classification::BlobPublicationCrashPoint;
use super::super::{
    BlobPublicationCrashBoundaryReport, BlobPublicationDenial,
    BlobPublicationReplayCounterSnapshot, BlobPublicationReplayedCrashEdge,
    BlobPublicationSessionCloseout,
};
use super::{
    reachability_staging::BlobReachabilityStaging, root_candidate::BlobRootCandidateForPublication,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationRecoveryEvidence {
    pub(crate) crash_point: BlobPublicationCrashPoint,
    pub(crate) evidence_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationPreWalReplayEvidence {
    pub(crate) operation_digest: String,
    pub(crate) classification_digest: String,
    pub(crate) replay_read_identity: String,
    pub(crate) counters: BlobPublicationReplayCounterSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobPublicationRecoveredState {
    DurableChunkNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    ChecksumAdmittedNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    ChunkTreeNodeDurableNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    RootCandidateNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    ReachabilityStagedNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    PublicationRecordReplayableNotVisible {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
    SessionClosedAwaitingVisibilityCommit {
        counters: super::super::BlobPublicationCounterSnapshot,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlobPublicationRecoveryReplay {
    pub(crate) evidence: BlobPublicationRecoveryEvidence,
    pub(crate) recovered_state: BlobPublicationRecoveredState,
}

impl BlobPublicationRecoveryEvidence {
    pub fn chunk_write_replayed(
        digest: &LogicalContentDigest,
        replay: BlobPublicationPreWalReplayEvidence,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::chunk_write_replayed(digest, replay)
    }

    pub fn checksum_admitted(
        digest: &LogicalContentDigest,
        replay: BlobPublicationPreWalReplayEvidence,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::checksum_admitted(digest, replay)
    }

    pub fn chunk_tree_node_durable(
        root: &ChunkTreeRoot,
        replay: BlobPublicationPreWalReplayEvidence,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::chunk_tree_node_durable(root, replay)
    }

    pub fn root_candidate(
        candidate: &BlobRootCandidateForPublication,
        replay: BlobPublicationPreWalReplayEvidence,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::root_candidate(candidate, replay)
    }

    pub fn reachability_staged(
        staged: &BlobReachabilityStaging,
        replay: BlobPublicationPreWalReplayEvidence,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::reachability_staged(staged, replay)
    }

    pub fn publication_record_replayable(
        report: &BlobPublicationCrashBoundaryReport,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::recovery_evidence::publication_record_replayable(report)
    }

    pub fn session_closed(closeout: &BlobPublicationSessionCloseout) -> Self {
        super::super::transitions::recovery_evidence::session_closed(closeout)
    }

    pub(crate) fn new(
        crash_point: BlobPublicationCrashPoint,
        evidence_digest: impl Into<String>,
    ) -> Self {
        Self {
            crash_point,
            evidence_digest: evidence_digest.into(),
        }
    }

    pub const fn crash_point(&self) -> BlobPublicationCrashPoint {
        self.crash_point
    }

    pub fn evidence_digest(&self) -> &str {
        &self.evidence_digest
    }
}

impl BlobPublicationPreWalReplayEvidence {
    pub fn from_chunk_write_replay(
        digest: &LogicalContentDigest,
        replay: &BlobPublicationReplayedCrashEdge,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::pre_wal_replay::from_chunk_write_replay(digest, replay)
    }

    pub fn from_checksum_admitted_replay(
        digest: &LogicalContentDigest,
        replay: &BlobPublicationReplayedCrashEdge,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::pre_wal_replay::from_checksum_admitted_replay(digest, replay)
    }

    pub fn from_chunk_tree_node_durable_replay(
        root: &ChunkTreeRoot,
        replay: &BlobPublicationReplayedCrashEdge,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::pre_wal_replay::from_chunk_tree_node_durable_replay(root, replay)
    }

    pub fn from_root_candidate_replay(
        candidate: &BlobRootCandidateForPublication,
        replay: &BlobPublicationReplayedCrashEdge,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::pre_wal_replay::from_root_candidate_replay(candidate, replay)
    }

    pub fn from_reachability_staged_replay(
        staged: &BlobReachabilityStaging,
        replay: &BlobPublicationReplayedCrashEdge,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::transitions::pre_wal_replay::from_reachability_staged_replay(staged, replay)
    }

    #[cfg(test)]
    pub(crate) fn chunk_write_recovery_operation_digest(
        digest: &LogicalContentDigest,
    ) -> super::super::evidence::BlobPublicationRecoveryOperationDigest {
        super::super::evidence::operation_digest::chunk_write_recovery_operation_digest(digest)
    }

    #[cfg(test)]
    pub(crate) fn checksum_admitted_recovery_operation_digest(
        digest: &LogicalContentDigest,
    ) -> super::super::evidence::BlobPublicationRecoveryOperationDigest {
        super::super::evidence::operation_digest::checksum_recovery_operation_digest(digest)
    }

    #[cfg(test)]
    pub(crate) fn chunk_tree_node_durable_recovery_operation_digest(
        root: &ChunkTreeRoot,
    ) -> super::super::evidence::BlobPublicationRecoveryOperationDigest {
        super::super::evidence::operation_digest::chunk_tree_recovery_operation_digest(root)
    }

    #[cfg(test)]
    pub(crate) fn root_candidate_recovery_operation_digest(
        candidate: &BlobRootCandidateForPublication,
    ) -> super::super::evidence::BlobPublicationRecoveryOperationDigest {
        super::super::evidence::operation_digest::root_candidate_recovery_operation_digest(
            candidate,
        )
    }

    #[cfg(test)]
    pub(crate) fn reachability_staged_recovery_operation_digest(
        staged: &BlobReachabilityStaging,
    ) -> super::super::evidence::BlobPublicationRecoveryOperationDigest {
        super::super::evidence::operation_digest::reachability_recovery_operation_digest(staged)
    }

    pub(crate) fn require_operation(
        self,
        expected_operation_digest: &super::super::evidence::BlobPublicationRecoveryOperationDigest,
    ) -> Result<Self, BlobPublicationDenial> {
        super::super::verification::pre_wal_replay::require_operation(
            self,
            expected_operation_digest,
        )
    }

    pub fn classification_digest(&self) -> &str {
        &self.classification_digest
    }

    pub fn replay_read_identity(&self) -> &str {
        &self.replay_read_identity
    }

    pub const fn counters(&self) -> BlobPublicationReplayCounterSnapshot {
        self.counters
    }
}

impl BlobPublicationRecoveryReplay {
    pub fn recover(evidence: BlobPublicationRecoveryEvidence) -> Self {
        super::super::transitions::recovery_replay::recover(evidence)
    }

    pub const fn crash_point(&self) -> BlobPublicationCrashPoint {
        self.evidence.crash_point()
    }

    pub const fn evidence(&self) -> &BlobPublicationRecoveryEvidence {
        &self.evidence
    }

    pub const fn recovered_state(&self) -> BlobPublicationRecoveredState {
        self.recovered_state
    }
}
