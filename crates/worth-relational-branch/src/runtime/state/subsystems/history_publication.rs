//! The branch ledger consumes prepared artifact accelerators after canonical position binding.
#[path = "history_publication/sequence.rs"]
pub mod sequence;
pub use sequence::PreparedCommitSequence;
use std::sync::Arc;

use crate::branch::reference::RelationalBranchReferenceCell;
use crate::branch::root::RelationalBranchRoot;
use crate::history::commit::RelationalCommitArtifact;
use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};

use super::HistorySubsystem;

pub struct PreparedVersionedArtifactPublication {
    pub commit_id: CommitId,
    pub commit_reference: RelationalCommitReceipt,
    pub branch_id: BranchId,
    pub next_cell: RelationalBranchReferenceCell,
    pub envelope: Arc<CanonicalCommitEnvelope>,
    pub root: Arc<RelationalBranchRoot>,
    pub artifact: Arc<RelationalCommitArtifact>,
    pub new_authoritative_bytes: u64,
    pub recovery_readmission: bool,
    pub sequence: PreparedCommitSequence,
}

/// Recovery-only versioned artifact installation input.
///
/// Live publication cannot construct or install this type through its
/// prepared-publication surface. Recovery preparation must first select the
/// `RecoveryTruth` validation sequence and explicitly wrap that result here.
pub struct PreparedRecoveredVersionedArtifactPublication {
    prepared: PreparedVersionedArtifactPublication,
}

pub struct PreparedVersionedArtifactAccelerators {
    commit_id: CommitId,
    commit_reference: RelationalCommitReceipt,
    branch_id: BranchId,
    root: Arc<RelationalBranchRoot>,
    artifact: Arc<RelationalCommitArtifact>,
    new_authoritative_bytes: u64,
    recovery_readmission: bool,
    sequence: PreparedCommitSequence,
}

impl PreparedVersionedArtifactPublication {
    pub fn into_canonical_and_accelerators(
        self,
    ) -> (
        RelationalBranchReferenceCell,
        Arc<RelationalBranchRoot>,
        Arc<CanonicalCommitEnvelope>,
        PreparedVersionedArtifactAccelerators,
    ) {
        let accelerator = PreparedVersionedArtifactAccelerators {
            commit_id: self.commit_id,
            commit_reference: self.commit_reference,
            branch_id: self.branch_id,
            root: Arc::clone(&self.root),
            artifact: self.artifact,
            new_authoritative_bytes: self.new_authoritative_bytes,
            recovery_readmission: self.recovery_readmission,
            sequence: self.sequence,
        };
        (self.next_cell, self.root, self.envelope, accelerator)
    }
}

impl PreparedRecoveredVersionedArtifactPublication {
    pub fn from_recovery_readmission(
        prepared: PreparedVersionedArtifactPublication,
    ) -> Result<Self, &'static str> {
        if !prepared.recovery_readmission {
            return Err("live prepared publication cannot enter recovery installation");
        }
        Ok(Self { prepared })
    }

    pub fn into_installation(
        self,
    ) -> (
        RelationalBranchReferenceCell,
        Arc<RelationalBranchRoot>,
        PreparedVersionedArtifactAccelerators,
    ) {
        let (cell, root, _, accelerators) = self.prepared.into_canonical_and_accelerators();
        (cell, root, accelerators)
    }
}

impl PreparedVersionedArtifactAccelerators {
    pub fn branch_id(&self) -> &BranchId {
        &self.branch_id
    }

    pub fn install(
        self,
        history: &HistorySubsystem,
        patch_position: crate::publication::patch::data::PatchStreamPosition,
    ) {
        let PreparedVersionedArtifactAccelerators {
            commit_id,
            commit_reference,
            branch_id,
            root,
            artifact,
            new_authoritative_bytes,
            recovery_readmission,
            sequence,
        } = self;
        // Preparation already accounted `canonical.payload_storage_bytes()`.
        // Add only the real original Memory frame admitted at position binding.
        let new_authoritative_bytes = new_authoritative_bytes
            .checked_add(artifact.envelope_residency().original_frame_storage_bytes());
        history.with_ledger_mut(|ledger| {
            ledger.install_published_commit(
                commit_id,
                commit_reference,
                artifact,
                patch_position,
                recovery_readmission,
                sequence,
            );
        });
        history.record_root_publication(&branch_id, &root, new_authoritative_bytes);
    }
}
