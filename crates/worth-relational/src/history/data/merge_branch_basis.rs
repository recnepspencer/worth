use std::fmt::Write;
#[cfg(test)]
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::{BranchId, CommitId, RelationalCommitReceipt};
mod ancestors;
pub use ancestors::MergeBasisAncestors;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeBaseSelectionRule {
    MaxCommitIdCommonAncestor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedMergeBase {
    pub(crate) rule: MergeBaseSelectionRule,
    pub(crate) commit: RelationalCommitReceipt,
    pub(crate) supporting_left_ancestors: MergeBasisAncestors,
    pub(crate) supporting_right_ancestors: MergeBasisAncestors,
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
        let mut digest = Sha256::new();
        canonical_commit_reference_bytes(&mut digest, &self.source_head);
        canonical_commit_reference_bytes(&mut digest, &self.target_head);
        canonical_selection_rule_bytes(&mut digest, self.merge_base.rule);
        canonical_commit_reference_bytes(&mut digest, &self.merge_base.commit);
        canonical_commit_ids_bytes(
            &mut digest,
            self.merge_base.supporting_left_ancestors.as_ref(),
        );
        canonical_commit_ids_bytes(
            &mut digest,
            self.merge_base.supporting_right_ancestors.as_ref(),
        );
        sha256_hex(digest.finalize().into())
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

fn canonical_selection_rule_bytes(bytes: &mut Sha256, rule: MergeBaseSelectionRule) {
    match rule {
        MergeBaseSelectionRule::MaxCommitIdCommonAncestor => bytes.update(b"max"),
    }
    bytes.update([0xff]);
}

fn canonical_commit_reference_bytes(bytes: &mut Sha256, reference: &RelationalCommitReceipt) {
    write_u64(bytes, reference.commit_id.0);
    write_u64(bytes, reference.version_id.0);
    bytes.update(reference.branch_id.0.as_bytes());
    bytes.update([0xfe]);
    canonical_commit_ids_bytes(bytes, reference.parents.as_slice());
}

fn canonical_commit_ids_bytes(bytes: &mut Sha256, commit_ids: &[CommitId]) {
    write_u64(bytes, commit_ids.len() as u64);
    for commit_id in commit_ids {
        write_u64(bytes, commit_id.0);
    }
}

fn write_u64(bytes: &mut Sha256, mut value: u64) {
    let mut digits = [b'0'; 20];
    for digit in digits.iter_mut().rev() {
        *digit += (value % 10) as u8;
        value /= 10;
    }
    bytes.update(digits);
    bytes.update([0xfd]);
}

fn sha256_hex(digest: [u8; 32]) -> String {
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect(/* Proof: The destination is String and all formatted arguments are native integers; String::write_str always returns Ok and integer formatting introduces no fallible writer. */ "hex formatting should succeed");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::VersionId;

    #[test]
    fn streamed_merge_basis_digest_preserves_decimal_delimiters_and_ancestor_order() {
        let receipt = |id, branch: &str, parents| RelationalCommitReceipt {
            commit_id: CommitId(id),
            version_id: VersionId(id),
            branch_id: BranchId(branch.into()),
            parents,
        };
        let basis = RelationalMergeBranchBasis {
            source_head: receipt(u64::MAX, "source-λ", vec![CommitId(9), CommitId(2)]),
            target_head: receipt(0, "target", vec![]),
            merge_base: ResolvedMergeBase {
                rule: MergeBaseSelectionRule::MaxCommitIdCommonAncestor,
                commit: receipt(2, "base", vec![CommitId(0)]),
                supporting_left_ancestors: Arc::<[CommitId]>::from([CommitId(9), CommitId(2)])
                    .into(),
                supporting_right_ancestors: Arc::<[CommitId]>::from([CommitId(2)]).into(),
            },
        };
        let number = |bytes: &mut Vec<u8>, n| {
            bytes.extend(format!("{n:020}").as_bytes());
            bytes.push(0xfd);
        };
        let ids = |bytes: &mut Vec<u8>, values: &[CommitId]| {
            number(bytes, values.len() as u64);
            for value in values {
                number(bytes, value.0);
            }
        };
        let reference = |bytes: &mut Vec<u8>, value: &RelationalCommitReceipt| {
            number(bytes, value.commit_id.0);
            number(bytes, value.version_id.0);
            bytes.extend(value.branch_id.0.as_bytes());
            bytes.push(0xfe);
            ids(bytes, &value.parents);
        };
        let mut prior = Vec::new();
        reference(&mut prior, &basis.source_head);
        reference(&mut prior, &basis.target_head);
        prior.extend(b"max");
        prior.push(0xff);
        reference(&mut prior, &basis.merge_base.commit);
        ids(&mut prior, &basis.merge_base.supporting_left_ancestors);
        ids(&mut prior, &basis.merge_base.supporting_right_ancestors);
        assert_eq!(basis.basis_digest(), format!("{:x}", Sha256::digest(prior)));
    }
}
