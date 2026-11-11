//! Reconstructed merge-plan copies retain only their newly allocated backing.
use super::normalized_plan::{intent_bytes, reserve_copy};
use crate::history::data::{CanonicalCommitEnvelope, OrderedParentList};
use crate::transactions::data::{MergeCommitMutationPlan, TransactionCommitError};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

mod copy_storage;

#[derive(Debug)]
pub(crate) struct OwnedMergePlan {
    plan: MergeCommitMutationPlan,
    _memory: Option<ExecutionMemoryReservation>,
}
impl OwnedMergePlan {
    pub(crate) fn plan(&self) -> &MergeCommitMutationPlan {
        &self.plan
    }

    /// An existing ordinary producer moves its actual plan; no copy is made or
    /// retrospectively charged by this ownership handoff.
    pub(crate) fn from_ordinary_plan(plan: MergeCommitMutationPlan) -> Self {
        Self {
            plan,
            _memory: None,
        }
    }

    pub(crate) fn copy_recovered(
        envelope: &CanonicalCommitEnvelope,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Option<Self>, TransactionCommitError> {
        request
            .consult()
            .map_err(super::normalized_plan::request_denial)?;
        let Some(authority) = &envelope.merge_execution_authority else {
            return Ok(None);
        };
        if !authority.retains_consistent_proof_packet_authority() {
            return Ok(None);
        }
        let summary = &authority.execution_summary;
        let packet = summary.proof_packet();
        let bytes = intent_bytes(&envelope.merged_plan.merged_intents).and_then(|bytes| {
            bytes
                .checked_add(packet.request().target_branch().0.len())?
                .checked_add(packet.request().source_branch().0.len())?
                .checked_add(copy_storage::arc_slots::<crate::history::data::BranchId>(
                    envelope.merge_parent_branches.len(),
                )?)?
                .checked_add(
                    envelope
                        .merge_parent_branches
                        .iter()
                        .try_fold(0usize, |bytes, branch| bytes.checked_add(branch.0.len()))?,
                )?
                .checked_add(super::normalized_plan::slots::<
                    crate::history::data::CommitId,
                >(envelope.commit.parents.len())?)?
                .checked_add(copy_storage::arc_slots::<crate::history::data::CommitId>(
                    envelope.merge_base_commits.len(),
                )?)?
                .checked_add(copy_storage::summary(summary)?)
        });
        let memory = reserve_copy(request, bytes)?;
        Ok(Some(Self {
            plan: MergeCommitMutationPlan {
                transaction_id: envelope.merged_plan.transaction_id,
                target_branch: packet.request().target_branch().clone(),
                source_branch: packet.request().source_branch().clone(),
                merge_parent_branches: std::sync::Arc::from(
                    envelope.merge_parent_branches.as_slice(),
                ),
                requested_merge_parent_count: envelope.merge_parent_branches.len(),
                parent_commits: OrderedParentList::from_authoritative(
                    envelope.commit.parents.clone(),
                ),
                merge_base_commits: std::sync::Arc::from(envelope.merge_base_commits.as_slice()),
                merged_plan: envelope.merged_plan.clone(),
                structural_summary: authority.structural_summary.clone(),
                merge_execution_summary: summary.clone(),
                proof_token: crate::transactions::data::merge_commit_mutation_plan_token(),
            },
            _memory: memory,
        }))
    }
}
