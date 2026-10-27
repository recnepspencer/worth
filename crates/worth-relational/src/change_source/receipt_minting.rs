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
    /// a carried resident root for an exact selection, or one fallible indexed
    /// envelope demand for an ancestor, followed by a pass over its patches.
    /// Storage and request refusals retain their typed execution cause.
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
        partition_id: Option<PartitionId>,
    ) -> Result<RelationalChangeReceiptOutcome, crate::execution::RelationalExecutionDenialCause>
    {
        crate::execution::check_request(execution)?;
        if selected.runtime_instance_id() != self.runtime_instance_id() {
            return Ok(TransitionOutcome::Stale(
                RelationalChangeReceiptStale::RuntimeAuthority,
            ));
        }
        let exact = selected
            .observation()
            .selected_root()
            .canonical_envelope()
            .filter(|envelope| envelope.commit.commit_id == selected.commit_id())
            .cloned();
        let envelope = match exact {
            Some(envelope) => Some(envelope),
            None => self
                .history()
                .demand_commit_envelope(execution, selected.commit_id())?,
        };
        Ok(self.mint_resident_change_receipt(selected, partition_id, envelope))
    }

    fn mint_resident_change_receipt(
        &self,
        selected: RelationalSelectedCommit,
        partition_id: Option<PartitionId>,
        envelope: Option<std::sync::Arc<crate::history::data::CanonicalCommitEnvelope>>,
    ) -> RelationalChangeReceiptOutcome {
        let commit_id = selected.commit_id();
        let unresolved = begin_change_receipt(ChangeReceiptRequest {
            runtime_instance_id: self.runtime_instance_id(),
            commit_id,
            partition_id,
        });
        let Some(envelope) = envelope else {
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
