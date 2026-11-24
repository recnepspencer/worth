use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::diagnostics::data::{
    DiagnosticsScope, RelationalDiagnosticFields, RelationalDiagnosticValue,
    RelationalDiagnosticsProfile,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::publication::authority::diagnostic_emission::DiagnosticArtifactBuilder;
use crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::transactions::data::commit_log::CommitHistorySummary;
use crate::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

pub(crate) struct CommitHistoryPreparation<'a> {
    pub(crate) history: &'a RelationalPreparationHistory,
    pub(crate) profile: &'a RelationalDiagnosticsProfile,
    pub(crate) diagnostics: &'a RelationalDiagnosticArtifactStore,
}

pub(crate) struct ResolvedCommitHistory {
    pub(crate) commit_id: CommitId,
    pub(crate) branch_id: BranchId,
    pub(crate) previous_branch_head_version: Option<VersionId>,
    pub(crate) commit_reference: RelationalCommitReceipt,
    pub(crate) merge_base_commits: Vec<CommitId>,
    pub(crate) requested_merge_parent_count: usize,
    pub(crate) effective_merge_parent_count: usize,
    pub(crate) branch_basis: AdmittedRelationalBranchBasis,
}

impl ResolvedCommitHistory {
    pub(crate) fn summary(&self) -> CommitHistorySummary {
        CommitHistorySummary {
            target_branch: self.branch_basis.identity().branch_id().0.clone(),
            requested_merge_parent_count: self.requested_merge_parent_count,
            effective_merge_parent_count: self.effective_merge_parent_count,
            parent_count: self.commit_reference.parents.len(),
            merge_base_count: self.merge_base_commits.len(),
            had_previous_branch_head: self.previous_branch_head_version.is_some(),
        }
    }
}

pub(crate) fn resolve_commit_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    preparation: &CommitHistoryPreparation<'_>,
    basis: &AdmittedRelationalBranchBasis,
    merge_parents: &[AdmittedRelationalBranchBasis],
    version_id: VersionId,
    commit_id: CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    let branch_basis = basis.clone();
    let branch_id = branch_basis.identity().branch_id().clone();
    if !branch_basis.is_current() {
        return Err(TransactionCommitError::conflict(
            crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                    detail: "owner-issued branch binding is foreign or no longer current"
                        .to_owned(),
                },
            ),
        ));
    }
    let previous_branch_head_version = branch_basis.commit_identity().map(|head| head.version_id());
    let requested_merge_parent_count = merge_parents.len();
    let (parents, merge_base_commits) =
        resolve_parent_commits(execution, preparation, basis, merge_parents)?;
    let effective_merge_parent_count = parents
        .len()
        .saturating_sub(usize::from(previous_branch_head_version.is_some()));
    let commit_reference = RelationalCommitReceipt {
        commit_id,
        version_id,
        branch_id: branch_id.clone(),
        parents,
    };
    Ok(ResolvedCommitHistory {
        commit_id,
        branch_id,
        previous_branch_head_version,
        commit_reference,
        merge_base_commits,
        requested_merge_parent_count,
        effective_merge_parent_count,
        branch_basis,
    })
}

fn resolve_parent_commits(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    preparation: &CommitHistoryPreparation<'_>,
    basis: &AdmittedRelationalBranchBasis,
    merge_parents: &[AdmittedRelationalBranchBasis],
) -> Result<(Vec<CommitId>, Vec<CommitId>), TransactionCommitError> {
    use crate::transactions::data::outcomes::conflict_class::ConflictClass;
    let mut parents = Vec::new();
    let mut merge_bases = std::collections::BTreeSet::new();
    let branch_id = basis.identity().branch_id();
    let target_head = basis.commit_identity().map(|identity| identity.commit_id());
    parents.extend(target_head);
    for merge_binding in merge_parents {
        let merge_branch = merge_binding.identity().branch_id().clone();
        if !merge_binding.is_current() {
            return Err(merge_parent_conflict(
                preparation,
                merge_parents,
                branch_id,
                ConflictClass::InvalidMergeParent {
                    detail: format!("merge parent is foreign or stale: {}", merge_branch.0),
                },
            ));
        }
        if &merge_branch == branch_id {
            continue;
        }
        let head = merge_binding
            .commit_identity()
            .map(|identity| identity.commit_id())
            .ok_or_else(|| {
                merge_parent_conflict(
                    preparation,
                    merge_parents,
                    branch_id,
                    ConflictClass::InvalidMergeParent {
                        detail: format!("merge parent has no head: {}", merge_branch.0),
                    },
                )
            })?;
        if parents.contains(&head) {
            continue;
        }
        if target_head.is_some() {
            let inspection = preparation
                .history
                .demand_merge_inspection(execution, merge_binding, basis)
                .map_err(|failure| {
                    crate::authority::commit::preparation::storage_read_denial(basis, failure)
                })?
                .ok_or_else(|| {
                    merge_parent_conflict(
                        preparation,
                        merge_parents,
                        branch_id,
                        ConflictClass::MissingMergeBase {
                            detail: format!(
                                "merge parent {} has no common ancestor with {}",
                                merge_branch.0, branch_id.0
                            ),
                        },
                    )
                })?;
            if !inspection.conflicting_records.is_empty() {
                return Err(merge_parent_conflict(
                    preparation,
                    merge_parents,
                    branch_id,
                    ConflictClass::MergeConflictOverlap {
                        detail: format!(
                            "merge between {} and {} overlaps {:?}",
                            merge_branch.0, branch_id.0, inspection.conflicting_records
                        ),
                    },
                ));
            }
            merge_bases.extend(inspection.merge_base);
        }
        parents.push(head);
    }
    Ok((parents, merge_bases.into_iter().collect()))
}

fn merge_parent_conflict(
    preparation: &CommitHistoryPreparation<'_>,
    merge_parents: &[AdmittedRelationalBranchBasis],
    branch_id: &BranchId,
    class: crate::transactions::data::outcomes::conflict_class::ConflictClass,
) -> TransactionCommitError {
    let conflict = crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(class);
    let branches = merge_parents
        .iter()
        .map(|basis| basis.identity().branch_id().clone())
        .collect::<Vec<_>>();
    DiagnosticArtifactBuilder::new(
        preparation.profile,
        preparation.diagnostics,
        DiagnosticsScope::History,
    )
    .failure()
    .emit_entry(
        conflict.code,
        conflict.detail.clone(),
        merge_parent_resolution_failure_fields(branch_id, &branches),
    );
    TransactionCommitError::conflict(conflict)
}

pub(crate) fn resolve_commit_history_for_merge(
    basis: &AdmittedRelationalBranchBasis,
    merge_plan: &MergeCommitMutationPlan,
    version_id: VersionId,
    commit_id: CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    let branch_id = merge_plan.target_branch.clone();
    let requested_merge_parent_count = merge_plan.requested_merge_parent_count;
    let branch_basis = basis.clone();
    if branch_basis.identity().branch_id() != &branch_id {
        return Err(TransactionCommitError::conflict(
            crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                crate::transactions::data::outcomes::conflict_class::ConflictClass::InvalidMergeParent {
                    detail: format!(
                        "owner-issued branch binding does not match target branch: {}",
                        branch_id.0
                    ),
                },
            ),
        ));
    }
    if !branch_basis.is_current() {
        return Err(TransactionCommitError::conflict(
            crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                    detail: "owner-issued branch binding is foreign or no longer current"
                        .to_owned(),
                },
            ),
        ));
    }
    let previous_branch_head_version = branch_basis.commit_identity().map(|head| head.version_id());
    let parents = merge_plan.parent_commits.clone_inner();
    let merge_base_commits = merge_plan.merge_base_commits.iter().copied().collect();
    let effective_merge_parent_count = parents
        .len()
        .saturating_sub(usize::from(previous_branch_head_version.is_some()));
    let commit_reference = RelationalCommitReceipt {
        commit_id,
        version_id,
        branch_id: branch_id.clone(),
        parents,
    };
    Ok(ResolvedCommitHistory {
        commit_id,
        branch_id,
        previous_branch_head_version,
        commit_reference,
        merge_base_commits,
        requested_merge_parent_count,
        effective_merge_parent_count,
        branch_basis,
    })
}

fn merge_parent_resolution_failure_fields(
    branch_id: &BranchId,
    merge_parent_branches: &[BranchId],
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "branch_id",
            RelationalDiagnosticValue::string(branch_id.0.clone()),
        ),
        (
            "merge_parent_branches",
            RelationalDiagnosticValue::array(
                merge_parent_branches
                    .iter()
                    .map(|branch| RelationalDiagnosticValue::string(branch.0.clone())),
            ),
        ),
    ])
    .into()
}
