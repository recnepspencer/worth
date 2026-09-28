use super::consistency::{RelationalChangeConsistencyDenial, RelationalChangeConsistencyWork};
use super::receipt_witness::ChangeReceiptProof;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};
use crate::identity_authority::{RelationalCommitIdentityKind, RelationalSourceTruthAuthorityIdentity};
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;

/// What minting a change receipt can come to.
///
/// The families keep their meaning across consumers: `Denied` is a change
/// that is not self-consistent, `Deferred` a commit that is not visible yet,
/// and `Stale` a selection that no longer holds.
pub type RelationalChangeReceiptOutcome = worth_proof::TransitionOutcome<
    RelationalChangeReceipt,
    RelationalChangeConsistencyDenial,
    RelationalChangeReceiptDeferred,
    RelationalChangeReceiptStale,
>;

/// The commit is committed but not yet visible to publication. Asking again
/// later can succeed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalChangeReceiptDeferred {
    /// The commit has not reached the canonical change stream yet.
    CommitVisibilityPending,
}

/// The selection no longer holds, so asking again cannot succeed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RelationalChangeReceiptStale {
    /// The commit was selected by a different runtime.
    RuntimeAuthority,
    /// The commit is no longer retained.
    CommitNotRetained,
}

/// Relational's proof that one retained commit's change is self-consistent,
/// together with that change.
///
/// Only [`RelationalRuntime::mint_change_receipt`](crate::runtime::RelationalRuntime::mint_change_receipt)
/// mints it, after the consistency checks pass, so holding one proves they
/// did. The patch is canonical and already narrowed to the requested
/// partition. The receipt names no consumer concept; a consumer supplies its
/// own when it lowers the receipt.
#[must_use = "a change receipt proves one commit's change; lower it or drop it deliberately"]
pub struct RelationalChangeReceipt {
    pub(super) patch: PublishedAuthoritativePatchEnvelope,
    pub(super) records_examined: u64,
    pub(super) records_filtered_out: u64,
    pub(super) consistency: RelationalChangeConsistencyWork,
    pub(super) proof: ChangeReceiptProof,
    pub(super) commit_identity:
        RelationalSourceTruthAuthorityIdentity<u64, RelationalCommitIdentityKind>,
}

impl RelationalChangeReceipt {
    /// The commit's canonical patch, narrowed to the requested partition.
    pub fn patch(&self) -> &PublishedAuthoritativePatchEnvelope {
        &self.patch
    }

    /// The runtime that minted the receipt.
    pub fn runtime_instance_id(&self) -> u64 {
        self.proof.payload().runtime_instance_id
    }

    /// The commit the change belongs to.
    pub fn commit_id(&self) -> CommitId {
        self.proof.payload().commit_id
    }

    /// The truth version the commit produced.
    pub fn version_id(&self) -> VersionId {
        self.proof.strong_basis().value().version_id
    }

    /// The branch whose retained observation selected the commit.
    pub fn selected_branch_id(&self) -> &BranchId {
        &self.proof.strong_basis().value().selected_branch_id
    }

    /// The branch the commit was written on. It differs from the selected
    /// branch when the commit is an ancestor inherited by a fork.
    pub fn authoring_branch_id(&self) -> &BranchId {
        &self.proof.strong_basis().value().authoring_branch_id
    }

    /// The partition the patch was narrowed to, or `None` for every
    /// partition.
    pub fn partition_id(&self) -> Option<PartitionId> {
        self.proof.payload().partition_id
    }

    /// Record patches in the commit before partition narrowing.
    pub const fn records_examined(&self) -> u64 {
        self.records_examined
    }

    /// Record patches partition narrowing dropped.
    pub const fn records_filtered_out(&self) -> u64 {
        self.records_filtered_out
    }

    /// The work the consistency checks did.
    pub const fn consistency_work(&self) -> RelationalChangeConsistencyWork {
        self.consistency
    }

    /// The source-truth identity Relational admitted for the commit.
    pub fn commit_identity(
        &self,
    ) -> &RelationalSourceTruthAuthorityIdentity<u64, RelationalCommitIdentityKind> {
        &self.commit_identity
    }
}

impl std::fmt::Debug for RelationalChangeReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalChangeReceipt")
            .field("runtime_instance_id", &self.runtime_instance_id())
            .field("commit_id", &self.commit_id())
            .field("partition_id", &self.partition_id())
            .field("records", &self.patch.authoritative_record_patches.len())
            .finish_non_exhaustive()
    }
}
