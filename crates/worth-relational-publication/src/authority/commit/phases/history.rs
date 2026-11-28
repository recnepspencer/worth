use contracts::history::data::commit_receipt::RelationalCommitReceipt;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::commit_log::CommitHistorySummary;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisOwnerPort as _,
};
use worth_relational_contracts::internal_port as contracts;

pub(crate) struct ResolvedCommitHistory {
    pub(crate) commit_id: CommitId,
    pub(crate) branch_id: BranchId,
    pub(crate) previous_branch_head_version: Option<VersionId>,
    pub(crate) commit_reference: RelationalCommitReceipt,
    pub(crate) branch_basis: AdmittedRelationalBranchBasis,
}

impl ResolvedCommitHistory {
    pub(crate) fn summary(&self) -> CommitHistorySummary {
        CommitHistorySummary {
            target_branch: self.branch_basis.identity().branch_id().0.clone(),
            requested_merge_parent_count: 0,
            effective_merge_parent_count: 0,
            parent_count: self.commit_reference.parents.len(),
            merge_base_count: 0,
            had_previous_branch_head: self.previous_branch_head_version.is_some(),
        }
    }
}

pub(crate) fn resolve_commit_history(
    basis: &AdmittedRelationalBranchBasis,
    version_id: VersionId,
    commit_id: CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    let branch_basis = basis.clone();
    let branch_id = branch_basis.identity().branch_id().clone();
    if !branch_basis.is_current() {
        return Err(TransactionCommitError::conflict(
            contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                contracts::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                    detail: "owner-issued branch binding is foreign or no longer current"
                        .to_owned(),
                },
            ),
        ));
    }
    let previous_branch_head_version = branch_basis.commit_identity().map(|head| head.version_id());
    let commit_reference = RelationalCommitReceipt {
        commit_id,
        version_id,
        branch_id: branch_id.clone(),
        parents: branch_basis
            .commit_identity()
            .map(|identity| identity.commit_id())
            .into_iter()
            .collect(),
    };
    Ok(ResolvedCommitHistory {
        commit_id,
        branch_id,
        previous_branch_head_version,
        commit_reference,
        branch_basis,
    })
}
