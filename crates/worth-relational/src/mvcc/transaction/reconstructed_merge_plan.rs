//! Reconstructed merge-plan copies retain only their newly allocated backing.
use super::normalized_plan::{intent_bytes, reserve_copy};
use crate::history::data::{CanonicalCommitEnvelope, OrderedParentList};
use crate::transactions::data::outcomes::plan_artifacts::{
    merge_commit_mutation_plan_token, MergeCommitMutationPlan, PublishedMergeExecutionAuthority,
};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::assemble_merge_commit_mutation_plan;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlanParts;
use worth_relational_contracts::internal_port::transactions::data::outcomes::plan_artifacts::PublishedMergeExecutionAuthorityOwnerPort as _;

mod copy_storage;

/// The recorded executable merge material borrowed from its own canonical commit.
#[derive(Debug)]
pub(crate) struct RecordedMergePlanSource<'envelope> {
    envelope: &'envelope CanonicalCommitEnvelope,
    authority: &'envelope PublishedMergeExecutionAuthority,
}
impl<'envelope> RecordedMergePlanSource<'envelope> {
    pub(crate) fn from_envelope(envelope: &'envelope CanonicalCommitEnvelope) -> Option<Self> {
        let authority = envelope.merge_execution_authority.as_ref()?;
        authority
            .retains_consistent_proof_packet_authority()
            .then_some(Self {
                envelope,
                authority,
            })
    }
}

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
        source: RecordedMergePlanSource<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, TransactionCommitError> {
        let RecordedMergePlanSource {
            envelope,
            authority,
        } = source;
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
        Ok(Self {
            plan: assemble_merge_commit_mutation_plan(MergeCommitMutationPlanParts {
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
                proof_token: merge_commit_mutation_plan_token(),
            }),
            _memory: memory,
        })
    }
}
