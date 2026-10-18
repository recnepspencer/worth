use std::sync::Arc;

use crate::branch::RelationalBranchReferenceCell;
use crate::branch::RelationalBranchRoot;
use crate::history::data::{BranchId, CanonicalCommitEnvelope, CommitId, RelationalCommitReceipt};
use crate::history::RelationalCommitArtifact;

use super::HistorySubsystem;

pub(crate) struct PreparedVersionedArtifactPublication {
    pub(crate) commit_id: CommitId,
    pub(crate) commit_reference: RelationalCommitReceipt,
    pub(crate) branch_id: BranchId,
    pub(crate) next_cell: RelationalBranchReferenceCell,
    pub(crate) envelope: Arc<CanonicalCommitEnvelope>,
    pub(crate) root: Arc<RelationalBranchRoot>,
    pub(crate) artifact: RelationalCommitArtifact,
    pub(crate) new_authoritative_bytes: u64,
    pub(crate) recovery_readmission: bool,
}

/// Recovery-only versioned artifact installation input.
///
/// Live publication cannot construct or install this type through its
/// prepared-publication surface. Recovery preparation must first select the
/// `RecoveryTruth` validation sequence and explicitly wrap that result here.
pub(crate) struct PreparedRecoveredVersionedArtifactPublication {
    prepared: PreparedVersionedArtifactPublication,
}

pub(crate) struct PreparedVersionedArtifactAccelerators {
    commit_id: CommitId,
    commit_reference: RelationalCommitReceipt,
    branch_id: BranchId,
    envelope: Arc<CanonicalCommitEnvelope>,
    root: Arc<RelationalBranchRoot>,
    artifact: RelationalCommitArtifact,
    new_authoritative_bytes: u64,
    recovery_readmission: bool,
}

impl PreparedVersionedArtifactPublication {
    pub(crate) fn into_canonical_and_accelerators(
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
            envelope: Arc::clone(&self.envelope),
            root: Arc::clone(&self.root),
            artifact: self.artifact,
            new_authoritative_bytes: self.new_authoritative_bytes,
            recovery_readmission: self.recovery_readmission,
        };
        (self.next_cell, self.root, self.envelope, accelerator)
    }
}

impl PreparedRecoveredVersionedArtifactPublication {
    pub(crate) fn from_recovery_readmission(
        prepared: PreparedVersionedArtifactPublication,
    ) -> Result<Self, &'static str> {
        if !prepared.recovery_readmission {
            return Err("live prepared publication cannot enter recovery installation");
        }
        Ok(Self { prepared })
    }

    fn into_prepared(self) -> PreparedVersionedArtifactPublication {
        self.prepared
    }
}

impl HistorySubsystem {
    pub(crate) fn install_prepared_recovered_versioned_artifact(
        &self,
        prepared: PreparedRecoveredVersionedArtifactPublication,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), String> {
        let prepared = prepared.into_prepared();
        let (mut next_cell, root, _, accelerators) = prepared.into_canonical_and_accelerators();
        next_cell.install_root(root);
        let branch_id = accelerators.branch_id.clone();
        let current_cell = self.branch_cell(&branch_id).ok_or_else(|| {
            format!(
                "prepared recovery branch `{}` is not registered",
                branch_id.0
            )
        })?;
        let previous_root = current_cell.root().ok_or_else(|| {
            format!(
                "prepared recovery branch `{}` has no retained head",
                branch_id.0
            )
        })?;
        let mut head_retirement = self
            .reserve_branch_head_retirement(
                current_cell.identity(),
                &previous_root,
                current_cell.head_retention(),
            )
            .map_err(|denial| format!("recovery head replacement denied: {denial:?}"))?;
        let next_root = next_cell
            .root()
            .expect("prepared recovered publication installed its captured root");
        self.install_prepared_versioned_accelerators(accelerators, positioned.position());
        // Recovery truth still installs a complete reconstructed owner cell.
        self.branch_cell(&branch_id)
            .expect("prepared recovery branch remains registered")
            .replace_state(next_cell.state_snapshot());
        head_retirement.transfer_head(&previous_root, &next_root);
        head_retirement.replace_head(previous_root);
        Ok(())
    }

    pub(crate) fn install_prepared_versioned_accelerators(
        &self,
        prepared: PreparedVersionedArtifactAccelerators,
        patch_position: crate::publication::patch::data::PatchStreamPosition,
    ) {
        let PreparedVersionedArtifactAccelerators {
            commit_id,
            commit_reference,
            branch_id,
            envelope,
            root,
            artifact,
            new_authoritative_bytes,
            recovery_readmission,
        } = prepared;
        self.with_ledger_mut(|ledger| {
            ledger.install_published_commit(
                commit_id,
                commit_reference,
                envelope,
                artifact,
                patch_position,
                recovery_readmission,
            );
        });
        self.record_root_publication(&branch_id, &root, new_authoritative_bytes);
    }
}
