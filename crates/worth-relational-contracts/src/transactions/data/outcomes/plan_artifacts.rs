use crate::history::data::commit_receipt::OrderedParentList;
use crate::history::data::coordinates::{BranchId, CommitId};
use crate::history::data::merge_branch_basis::RelationalMergeBranchBasis;
use crate::internal_port::merge::data::artifacts::strategy_witness::RelationalMergeStrategyWitnessOwnerPort as _;
use crate::merge::data::artifacts::correspondence_witness::RelationalMergeCorrespondenceWitness;
use crate::merge::data::artifacts::proof_packet::RelationalMergeProofPacket;
use crate::merge::data::artifacts::schema_reconciliation_witness::RelationalSchemaReconciliationWitness;
use crate::merge::data::artifacts::strategy_witness::RelationalMergeStrategyWitness;
use crate::merge::data::requests::normalized::NormalizedRelationalMergeRequest;
use crate::transactions::data::intents::mutation_intent::MutationIntent;
use crate::transactions::data::primitives::{RecordRef, TransactionId};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergedCommitPlan {
    pub transaction_id: TransactionId,
    pub merged_intents: Vec<MutationIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionStructuralSummary {
    pub executed_record_count: usize,
    pub adopted_source_record_count: usize,
    pub preserved_shared_record_count: usize,
    pub reconciled_record_count: usize,
    pub converged_deleted_on_both_sides_count: usize,
    pub deleted_on_both_sides_lineage_unchanged_count: usize,
    pub emitted_mutation_intent_count: usize,
    pub emitted_entity_create_count: usize,
    pub emitted_relation_create_count: usize,
    pub emitted_entity_update_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeExecutionSummary {
    pub request: NormalizedRelationalMergeRequest,
    pub branch_basis: RelationalMergeBranchBasis,
    pub correspondence_witness: RelationalMergeCorrespondenceWitness,
    pub schema_reconciliation_witness: RelationalSchemaReconciliationWitness,
    pub strategy_witness: RelationalMergeStrategyWitness,
    pub proof_packet: RelationalMergeProofPacket,
    pub target_head_commit_id: CommitId,
    pub source_head_commit_id: CommitId,
    pub merge_base_commit_id: CommitId,
    pub executed_record_count: usize,
    pub adopted_source_record_count: usize,
    pub preserved_shared_record_count: usize,
    pub reconciled_record_count: usize,
    pub converged_deleted_on_both_sides_count: usize,
    pub deleted_on_both_sides_lineage_unchanged_count: usize,
    pub emitted_mutation_intent_count: usize,
    pub diagnostics_digest: String,
    pub execution_digest: String,
}

impl MergeExecutionSummary {
    pub fn proof_packet(&self) -> &RelationalMergeProofPacket {
        &self.proof_packet
    }

    pub fn correspondence_witness(&self) -> &RelationalMergeCorrespondenceWitness {
        &self.correspondence_witness
    }

    pub fn schema_reconciliation_witness(&self) -> &RelationalSchemaReconciliationWitness {
        &self.schema_reconciliation_witness
    }

    pub fn strategy_witness(&self) -> &RelationalMergeStrategyWitness {
        &self.strategy_witness
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MergeCommitMutationPlan {
    pub transaction_id: TransactionId,
    pub target_branch: BranchId,
    pub source_branch: BranchId,
    pub merge_parent_branches: Arc<[BranchId]>,
    pub requested_merge_parent_count: usize,
    pub parent_commits: OrderedParentList,
    pub merge_base_commits: Arc<[CommitId]>,
    pub merged_plan: MergedCommitPlan,
    pub structural_summary: MergeExecutionStructuralSummary,
    pub merge_execution_summary: MergeExecutionSummary,
    #[serde(
        skip_serializing,
        skip_deserializing,
        default = "merge_commit_mutation_plan_token"
    )]
    pub(crate) proof_token: MergeCommitMutationPlanToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishedMergeExecutionAuthority {
    pub execution_summary: MergeExecutionSummary,
    pub structural_summary: MergeExecutionStructuralSummary,
}

impl PublishedMergeExecutionAuthority {
    pub fn from_merge_plan(plan: &MergeCommitMutationPlan) -> Self {
        Self {
            execution_summary: plan.merge_execution_summary.clone(),
            structural_summary: plan.structural_summary.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MergeCommitMutationPlanToken;

pub fn merge_commit_mutation_plan_token() -> MergeCommitMutationPlanToken {
    MergeCommitMutationPlanToken
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoritativeApplyPlan {
    pub transaction_id: TransactionId,
    pub version_id: crate::identity::data::VersionId,
    pub merged_intents: Vec<MutationIntent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UndoRecord {
    pub record: RecordRef,
    pub detail: String,
}

/// Operations used by the Relational owners.
pub trait MergeExecutionSummaryOwnerPort: Sized {
    fn retains_consistent_proof_packet_authority(&self) -> bool;
}

impl MergeExecutionSummaryOwnerPort for MergeExecutionSummary {
    fn retains_consistent_proof_packet_authority(&self) -> bool {
        let packet = self.proof_packet();
        self.request == *packet.request()
            && self.branch_basis == *packet.branch_basis()
            && self.correspondence_witness.request_digest() == self.request.request_digest()
            && self.correspondence_witness.branch_basis_digest() == self.branch_basis.basis_digest()
            && packet.correspondence_witness_digest()
                == self.correspondence_witness.witness_digest()
            && self.schema_reconciliation_witness.request_digest() == self.request.request_digest()
            && self.schema_reconciliation_witness.branch_basis_digest()
                == self.branch_basis.basis_digest()
            && packet.schema_reconciliation_witness_digest()
                == self.schema_reconciliation_witness.witness_digest()
            && self.strategy_witness.retains_honest_truth()
            && self.strategy_witness.request_digest() == self.request.request_digest()
            && self.strategy_witness.branch_basis_digest() == self.branch_basis.basis_digest()
            && packet.strategy_witness_digest() == self.strategy_witness.witness_digest()
            && self.execution_digest == packet.execution_digest()
            && self.target_head_commit_id == self.branch_basis.target_head().commit_id
            && self.source_head_commit_id == self.branch_basis.source_head().commit_id
            && self.merge_base_commit_id == self.branch_basis.merge_base().commit().commit_id
    }
}

/// Operations used by the Relational owners.
pub trait PublishedMergeExecutionAuthorityOwnerPort: Sized {
    fn retains_consistent_proof_packet_authority(&self) -> bool;
}

impl PublishedMergeExecutionAuthorityOwnerPort for PublishedMergeExecutionAuthority {
    fn retains_consistent_proof_packet_authority(&self) -> bool {
        self.execution_summary
            .retains_consistent_proof_packet_authority()
    }
}

pub struct MergeCommitMutationPlanParts {
    pub transaction_id: TransactionId,
    pub target_branch: BranchId,
    pub source_branch: BranchId,
    pub merge_parent_branches: Arc<[BranchId]>,
    pub requested_merge_parent_count: usize,
    pub parent_commits: OrderedParentList,
    pub merge_base_commits: Arc<[CommitId]>,
    pub merged_plan: MergedCommitPlan,
    pub structural_summary: MergeExecutionStructuralSummary,
    pub merge_execution_summary: MergeExecutionSummary,
    pub proof_token: MergeCommitMutationPlanToken,
}

pub fn assemble_merge_commit_mutation_plan(
    parts: MergeCommitMutationPlanParts,
) -> MergeCommitMutationPlan {
    MergeCommitMutationPlan {
        transaction_id: parts.transaction_id,
        target_branch: parts.target_branch,
        source_branch: parts.source_branch,
        merge_parent_branches: parts.merge_parent_branches,
        requested_merge_parent_count: parts.requested_merge_parent_count,
        parent_commits: parts.parent_commits,
        merge_base_commits: parts.merge_base_commits,
        merged_plan: parts.merged_plan,
        structural_summary: parts.structural_summary,
        merge_execution_summary: parts.merge_execution_summary,
        proof_token: parts.proof_token,
    }
}
