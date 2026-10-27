use std::fmt::Write;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{BranchId, CommitId, RelationalCommitReceipt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeBaseSelectionRule {
    MaxCommitIdCommonAncestor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedMergeBase {
    pub(crate) rule: MergeBaseSelectionRule,
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) supporting_left_ancestors: Arc<[CommitId]>,
    pub(crate) supporting_right_ancestors: Arc<[CommitId]>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationalMergeBranchBasis {
    pub(crate) source_head: RelationalCommitReceipt,
    pub(crate) target_head: RelationalCommitReceipt,
    pub(crate) merge_base: ResolvedMergeBase,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationalMergeBranchBasisDenial {
    SourceObservationDenied(crate::branch::RelationalBranchBasisDenial),
    TargetObservationDenied(crate::branch::RelationalBranchBasisDenial),
    MissingSourceHead {
        branch_id: BranchId,
    },
    MissingTargetHead {
        branch_id: BranchId,
    },
    MissingMergeBase {
        source_branch: BranchId,
        target_branch: BranchId,
    },
    /// The selected common ancestor has no indexed canonical commit receipt.
    MissingMergeBaseCommit {
        commit_id: CommitId,
    },
}

impl RelationalMergeBranchBasis {
    pub fn source_head(&self) -> &RelationalCommitReceipt {
        &self.source_head
    }

    pub fn target_head(&self) -> &RelationalCommitReceipt {
        &self.target_head
    }

    pub fn merge_base(&self) -> &ResolvedMergeBase {
        &self.merge_base
    }

    pub fn source_branch(&self) -> &BranchId {
        &self.source_head.branch_id
    }

    pub fn target_branch(&self) -> &BranchId {
        &self.target_head.branch_id
    }

    pub fn basis_digest(&self) -> String {
        sha256_hex(&self.canonical_digest_bytes())
    }

    fn canonical_digest_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        canonical_commit_reference_bytes(&mut bytes, &self.source_head);
        canonical_commit_reference_bytes(&mut bytes, &self.target_head);
        canonical_selection_rule_bytes(&mut bytes, self.merge_base.rule);
        canonical_commit_reference_bytes(&mut bytes, &self.merge_base.commit);
        canonical_commit_ids_bytes(
            &mut bytes,
            self.merge_base.supporting_left_ancestors.as_ref(),
        );
        canonical_commit_ids_bytes(
            &mut bytes,
            self.merge_base.supporting_right_ancestors.as_ref(),
        );
        bytes
    }
}

impl ResolvedMergeBase {
    pub fn rule(&self) -> MergeBaseSelectionRule {
        self.rule
    }

    pub fn commit(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub fn supporting_left_ancestors(&self) -> &[CommitId] {
        self.supporting_left_ancestors.as_ref()
    }

    pub fn supporting_right_ancestors(&self) -> &[CommitId] {
        self.supporting_right_ancestors.as_ref()
    }
}

fn canonical_selection_rule_bytes(bytes: &mut Vec<u8>, rule: MergeBaseSelectionRule) {
    match rule {
        MergeBaseSelectionRule::MaxCommitIdCommonAncestor => bytes.extend_from_slice(b"max"),
    }
    bytes.push(0xff);
}

fn canonical_commit_reference_bytes(bytes: &mut Vec<u8>, reference: &RelationalCommitReceipt) {
    write_u64(bytes, reference.commit_id.0);
    write_u64(bytes, reference.version_id.0);
    bytes.extend_from_slice(reference.branch_id.0.as_bytes());
    bytes.push(0xfe);
    canonical_commit_ids_bytes(bytes, reference.parents.as_slice());
}

fn canonical_commit_ids_bytes(bytes: &mut Vec<u8>, commit_ids: &[CommitId]) {
    write_u64(bytes, commit_ids.len() as u64);
    for commit_id in commit_ids {
        write_u64(bytes, commit_id.0);
    }
}

fn write_u64(bytes: &mut Vec<u8>, value: u64) {
    let mut buffer = String::new();
    write!(&mut buffer, "{value:020}").expect("u64 formatting should succeed");
    bytes.extend_from_slice(buffer.as_bytes());
    bytes.push(0xfd);
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("hex formatting should succeed");
    }
    output
}
