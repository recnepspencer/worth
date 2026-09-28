use worth_foundational::facade::admit_foundational_authority_identity;
use worth_proof::TransitionOutcome;

use super::commit_selection::RelationalSelectedCommit;
use super::partition_projection::project_patch_partition;
use super::receipt::{
    RelationalChangeReceipt, RelationalChangeReceiptDeferred, RelationalChangeReceiptOutcome,
    RelationalChangeReceiptStale,
};
use super::receipt_witness::{
    admit_consistent_change, begin_change_receipt, resolve_change_receipt, ChangeReceiptBasis,
    ChangeReceiptRequest,
};
use crate::capabilities::CommitEnvelopeSource;
use crate::identity::data::PartitionId;
use crate::identity_authority::relational_source_truth_authority;
use crate::publication::patch::data::PublishedAuthoritativePatchEnvelope;
use crate::runtime::RelationalRuntime;

impl RelationalRuntime {
    /// Mint the change receipt for the commit `selected` names, narrowed to
    /// `partition_id` when one is given.
    ///
    /// The receipt holds the commit's canonical patch, and minting it runs
    /// the consistency checks, so a receipt is proof they passed. The cost is
    /// one retained-envelope lookup plus a pass over the commit's record
    /// patches.
    ///
    /// # Outcomes
    ///
    /// - `Stale(RuntimeAuthority)` when another runtime made the selection.
    /// - `Deferred(CommitVisibilityPending)` when the commit has not reached
    ///   the canonical change stream yet.
    /// - `Stale(CommitNotRetained)` when the commit is no longer retained.
    /// - `Denied` when the change is not self-consistent.
    pub fn mint_change_receipt(
        &self,
        selected: RelationalSelectedCommit,
        partition_id: Option<PartitionId>,
    ) -> RelationalChangeReceiptOutcome {
        if selected.runtime_instance_id() != self.runtime_instance_id() {
            return TransitionOutcome::Stale(RelationalChangeReceiptStale::RuntimeAuthority);
        }
        let commit_id = selected.commit_id();
        let unresolved = begin_change_receipt(ChangeReceiptRequest {
            runtime_instance_id: self.runtime_instance_id(),
            commit_id,
            partition_id,
        });
        let Some(envelope) = self.commit_envelope(commit_id) else {
            return if commit_id >= self.history().next_commit_id() {
                TransitionOutcome::Deferred(
                    RelationalChangeReceiptDeferred::CommitVisibilityPending,
                )
            } else {
                TransitionOutcome::Stale(RelationalChangeReceiptStale::CommitNotRetained)
            };
        };
        let Some(position) = self
            .history()
            .canonical_stream_position(envelope.commit.commit_id)
        else {
            return TransitionOutcome::Deferred(
                RelationalChangeReceiptDeferred::CommitVisibilityPending,
            );
        };
        let resolved = resolve_change_receipt(
            unresolved,
            ChangeReceiptBasis {
                version_id: envelope.commit.version_id,
                selected_branch_id: selected.observation().identity().branch_id().clone(),
                authoring_branch_id: envelope.commit.branch_id.clone(),
            },
        );
        let projection = project_patch_partition(
            PublishedAuthoritativePatchEnvelope::from_canonical(position, &envelope.patch),
            partition_id,
        );
        let patch = projection.patch.canonicalized();
        let consistency = match patch.check_change_consistency() {
            Ok(work) => work,
            Err(denial) => return TransitionOutcome::Denied(denial),
        };
        TransitionOutcome::Success(RelationalChangeReceipt {
            patch,
            records_examined: projection.records_examined,
            records_filtered_out: projection.records_filtered_out,
            consistency,
            proof: admit_consistent_change(resolved),
            commit_identity: admit_foundational_authority_identity(
                commit_id.0,
                relational_source_truth_authority(),
            ),
        })
    }
}
