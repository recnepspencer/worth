mod retained_charge;

use serde::{Deserialize, Serialize};

use crate::logic::transaction::runtime::state::merge::{
    BranchMergeRequestScopeFamily, ScopedMergeCandidateBreadthSummary, ScopedMergeProofPacket,
    SignalMergeStrategyWitness,
};
use crate::logic::transaction::runtime::state::{
    SignalBranchBasis, SignalBranchHeadPosture, SignalBranchRestorePosture,
};
use crate::state::{SignalBranchId, SignalSnapshotId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalMergeCompatibilityFactInventory {
    branch_id: SignalBranchId,
    snapshot_id: Option<SignalSnapshotId>,
    branch_basis_digest: String,
    branch_head_posture: SignalBranchHeadPosture,
    branch_restore_posture: SignalBranchRestorePosture,
    scope_family: BranchMergeRequestScopeFamily,
    declaration_digest: String,
    admitted_scope_digest: String,
    skipped_scope_digest: Option<String>,
    no_op_scope_digest: Option<String>,
    breadth_summary: ScopedMergeCandidateBreadthSummary,
    strategy_witness_digest: String,
    merge_strategy_digest: String,
    invalidation_strategy_digest: String,
    delivery_strategy_digest: String,
}

impl SignalMergeCompatibilityFactInventory {
    pub fn from_retained(
        branch_basis: &SignalBranchBasis,
        scoped_merge_proof: &ScopedMergeProofPacket,
        strategy_witness: &SignalMergeStrategyWitness,
    ) -> Self {
        Self {
            branch_id: branch_basis.branch_id(),
            snapshot_id: branch_basis.snapshot_id(),
            branch_basis_digest: branch_basis.basis_digest().to_owned(),
            branch_head_posture: branch_basis.head_posture().clone(),
            branch_restore_posture: branch_basis.restore_posture().clone(),
            scope_family: scoped_merge_proof.scope_family(),
            declaration_digest: scoped_merge_proof.declaration_digest().to_owned(),
            admitted_scope_digest: scoped_merge_proof.admitted_scope_digest().to_owned(),
            skipped_scope_digest: scoped_merge_proof
                .skipped_scope_digest()
                .map(ToOwned::to_owned),
            no_op_scope_digest: scoped_merge_proof
                .no_op_scope_digest()
                .map(ToOwned::to_owned),
            breadth_summary: scoped_merge_proof.breadth_summary().clone(),
            strategy_witness_digest: strategy_witness.witness_digest().to_owned(),
            merge_strategy_digest: strategy_witness.merge_strategy_digest().to_owned(),
            invalidation_strategy_digest: strategy_witness
                .invalidation_strategy_digest()
                .to_owned(),
            delivery_strategy_digest: strategy_witness.delivery_strategy_digest().to_owned(),
        }
    }

    pub fn branch_id(&self) -> SignalBranchId {
        self.branch_id
    }

    pub fn snapshot_id(&self) -> Option<SignalSnapshotId> {
        self.snapshot_id
    }

    pub fn branch_basis_digest(&self) -> &str {
        &self.branch_basis_digest
    }

    pub fn branch_head_posture(&self) -> &SignalBranchHeadPosture {
        &self.branch_head_posture
    }

    pub fn branch_restore_posture(&self) -> &SignalBranchRestorePosture {
        &self.branch_restore_posture
    }

    pub fn scope_family(&self) -> BranchMergeRequestScopeFamily {
        self.scope_family
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn admitted_scope_digest(&self) -> &str {
        &self.admitted_scope_digest
    }

    pub fn skipped_scope_digest(&self) -> Option<&str> {
        self.skipped_scope_digest.as_deref()
    }

    pub fn no_op_scope_digest(&self) -> Option<&str> {
        self.no_op_scope_digest.as_deref()
    }

    pub fn breadth_summary(&self) -> &ScopedMergeCandidateBreadthSummary {
        &self.breadth_summary
    }

    pub fn strategy_witness_digest(&self) -> &str {
        &self.strategy_witness_digest
    }

    pub fn merge_strategy_digest(&self) -> &str {
        &self.merge_strategy_digest
    }

    pub fn invalidation_strategy_digest(&self) -> &str {
        &self.invalidation_strategy_digest
    }

    pub fn delivery_strategy_digest(&self) -> &str {
        &self.delivery_strategy_digest
    }
}
