use serde::{Deserialize, Serialize};

use crate::transactions::data::{
    CommitConflict, CommitPreparationError, MergedCommitPlan, ProvenanceCompleteBulkMutationBatch,
    TransactionId,
};

use super::{
    CanonicalStrategyCommitRequest, CanonicalStrategyInputDigest, CanonicalStrategyOutputDigest,
    CommitStrategyDescriptorDigest, CommitStrategyId, StrategyExecutionDraft,
    StrategyMutationProgramDigest,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StrategyLoweringError {
    RequestExecutionMismatch { detail: String },
    MutationConflict(CommitConflict),
    Preparation(crate::transactions::data::TransactionCommitError),
}

impl StrategyLoweringError {
    pub(crate) fn mutation_conflict(conflict: CommitConflict) -> Self {
        Self::MutationConflict(conflict)
    }

    pub(crate) fn preparation(error: CommitPreparationError) -> Self {
        Self::Preparation(crate::transactions::data::TransactionCommitError::preparation(error))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyLoweringProvenance {
    strategy_id: CommitStrategyId,
    descriptor_digest: CommitStrategyDescriptorDigest,
    input_digest: CanonicalStrategyInputDigest,
    output_digest: CanonicalStrategyOutputDigest,
    mutation_program_digest: StrategyMutationProgramDigest,
}

impl StrategyLoweringProvenance {
    pub(crate) fn from_request_and_execution(
        request: &CanonicalStrategyCommitRequest,
        execution: &StrategyExecutionDraft,
    ) -> Self {
        Self {
            strategy_id: request.strategy_id(),
            descriptor_digest: request.descriptor_digest(),
            input_digest: request.canonical_input().digest(),
            output_digest: execution.output().digest(),
            mutation_program_digest: execution.mutation_program().digest(),
        }
    }

    pub fn strategy_id(&self) -> CommitStrategyId {
        self.strategy_id
    }

    pub fn descriptor_digest(&self) -> CommitStrategyDescriptorDigest {
        self.descriptor_digest
    }

    pub fn input_digest(&self) -> CanonicalStrategyInputDigest {
        self.input_digest
    }

    pub fn output_digest(&self) -> CanonicalStrategyOutputDigest {
        self.output_digest
    }

    pub fn mutation_program_digest(&self) -> StrategyMutationProgramDigest {
        self.mutation_program_digest
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrategyLoweringSummary {
    worker_batch_count: usize,
    total_intent_count: usize,
    touched_partition_count: usize,
    cross_partition_relation_count: usize,
    normalized_client_key_count: usize,
    lineage_transition_count: usize,
    projected_entity_record_reads: usize,
    projected_relation_record_reads: usize,
    projected_partition_reads: usize,
}

impl StrategyLoweringSummary {
    pub(crate) fn new(
        worker_batch_count: usize,
        total_intent_count: usize,
        touched_partition_count: usize,
        cross_partition_relation_count: usize,
        normalized_client_key_count: usize,
        lineage_transition_count: usize,
        projected_entity_record_reads: usize,
        projected_relation_record_reads: usize,
        projected_partition_reads: usize,
    ) -> Self {
        Self {
            worker_batch_count,
            total_intent_count,
            touched_partition_count,
            cross_partition_relation_count,
            normalized_client_key_count,
            lineage_transition_count,
            projected_entity_record_reads,
            projected_relation_record_reads,
            projected_partition_reads,
        }
    }

    pub fn worker_batch_count(&self) -> usize {
        self.worker_batch_count
    }

    pub fn total_intent_count(&self) -> usize {
        self.total_intent_count
    }

    pub fn touched_partition_count(&self) -> usize {
        self.touched_partition_count
    }

    pub fn cross_partition_relation_count(&self) -> usize {
        self.cross_partition_relation_count
    }

    pub fn normalized_client_key_count(&self) -> usize {
        self.normalized_client_key_count
    }

    pub fn lineage_transition_count(&self) -> usize {
        self.lineage_transition_count
    }

    pub fn projected_entity_record_reads(&self) -> usize {
        self.projected_entity_record_reads
    }

    pub fn projected_relation_record_reads(&self) -> usize {
        self.projected_relation_record_reads
    }

    pub fn projected_partition_reads(&self) -> usize {
        self.projected_partition_reads
    }
}

#[derive(Debug)]
pub struct LoweredStrategyCommitPlan {
    request: CanonicalStrategyCommitRequest,
    execution: StrategyExecutionDraft,
    transaction: crate::mvcc::BranchBoundRelationalTransaction,
    bulk_mutation_batch: Option<ProvenanceCompleteBulkMutationBatch>,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
    merged_plan: MergedCommitPlan,
    lowering_provenance: StrategyLoweringProvenance,
    lowering_summary: StrategyLoweringSummary,
}

impl LoweredStrategyCommitPlan {
    pub(crate) fn new(
        request: CanonicalStrategyCommitRequest,
        execution: StrategyExecutionDraft,
        transaction: crate::mvcc::BranchBoundRelationalTransaction,
        bulk_mutation_batch: Option<ProvenanceCompleteBulkMutationBatch>,
        selected_branch_state: crate::branch::SelectedRelationalBranchState,
        merged_plan: MergedCommitPlan,
        lowering_provenance: StrategyLoweringProvenance,
        lowering_summary: StrategyLoweringSummary,
    ) -> Self {
        Self {
            request,
            execution,
            transaction,
            bulk_mutation_batch,
            selected_branch_state,
            merged_plan,
            lowering_provenance,
            lowering_summary,
        }
    }

    pub fn request(&self) -> &CanonicalStrategyCommitRequest {
        &self.request
    }

    pub fn execution(&self) -> &StrategyExecutionDraft {
        &self.execution
    }

    pub fn transaction_id(&self) -> TransactionId {
        self.transaction.transaction_id()
    }

    pub(crate) fn transaction(&self) -> &crate::mvcc::BranchBoundRelationalTransaction {
        &self.transaction
    }

    pub(crate) fn into_validation_parts(
        self,
    ) -> (
        crate::mvcc::BranchBoundRelationalTransaction,
        Option<ProvenanceCompleteBulkMutationBatch>,
        crate::branch::SelectedRelationalBranchState,
        MergedCommitPlan,
    ) {
        (
            self.transaction,
            self.bulk_mutation_batch,
            self.selected_branch_state,
            self.merged_plan,
        )
    }

    pub fn bulk_mutation_batch(&self) -> Option<&ProvenanceCompleteBulkMutationBatch> {
        self.bulk_mutation_batch.as_ref()
    }

    pub fn merged_plan(&self) -> &MergedCommitPlan {
        &self.merged_plan
    }

    pub fn lowering_provenance(&self) -> StrategyLoweringProvenance {
        self.lowering_provenance
    }

    pub fn lowering_summary(&self) -> &StrategyLoweringSummary {
        &self.lowering_summary
    }
}
