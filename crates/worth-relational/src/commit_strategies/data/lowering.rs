//! A lowered strategy retains the exact transaction and selected branch state.
use super::artifact_basis::StrategyArtifactBasis;
use super::execution::execution_draft::StrategyExecutionDraft;
use super::lowering_meaning::{StrategyLoweringProvenance, StrategyLoweringSummary};
use super::request::CanonicalStrategyCommitRequest;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use crate::transactions::data::primitives::{ProvenanceCompleteBulkMutationBatch, TransactionId};

#[derive(Debug)]
pub struct LoweredStrategyCommitPlan {
    basis: StrategyArtifactBasis,
    transaction: crate::mvcc::BranchBoundRelationalTransaction,
    bulk_mutation_batch: Option<ProvenanceCompleteBulkMutationBatch>,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
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
            basis: StrategyArtifactBasis::new(
                request,
                execution,
                merged_plan,
                lowering_provenance,
                lowering_summary,
            ),
            transaction,
            bulk_mutation_batch,
            selected_branch_state,
        }
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
            self.basis.into_merged_plan(),
        )
    }

    pub fn bulk_mutation_batch(&self) -> Option<&ProvenanceCompleteBulkMutationBatch> {
        self.bulk_mutation_batch.as_ref()
    }
}

impl std::ops::Deref for LoweredStrategyCommitPlan {
    type Target = StrategyArtifactBasis;

    fn deref(&self) -> &Self::Target {
        &self.basis
    }
}
