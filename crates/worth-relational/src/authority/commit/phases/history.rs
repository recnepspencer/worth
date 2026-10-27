use crate::branch::AdmittedRelationalBranchBasis;
use crate::diagnostics::data::{
    DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticFields,
    RelationalDiagnosticValue, RelationalDiagnosticsEntry,
};
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::mvcc::RelationalTransactionValidationInput;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    CommitHistorySummary, MergeCommitMutationPlan, TransactionCommitError,
};

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
    runtime: &RelationalPreparationRuntime,
    options: &RelationalTransactionValidationInput,
    version_id: VersionId,
    commit_id: CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    let branch_basis = options.basis().clone();
    let branch_id = branch_basis.identity().branch_id().clone();
    if !branch_basis.is_current() {
        return Err(TransactionCommitError::conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::StaleValidationBasis {
                    detail: "owner-issued branch binding is foreign or no longer current"
                        .to_owned(),
                },
            ),
        ));
    }
    let previous_branch_head_version = branch_basis.commit_identity().map(|head| head.version_id());
    let requested_merge_parent_count = options.merge_parent_bases().len();
    let (parents, merge_base_commits) = resolve_parent_commits(runtime, options, &branch_id)?;
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
    runtime: &RelationalPreparationRuntime,
    options: &RelationalTransactionValidationInput,
    branch_id: &BranchId,
) -> Result<(Vec<CommitId>, Vec<CommitId>), TransactionCommitError> {
    let mut parents = Vec::new();
    let mut merge_bases = std::collections::BTreeSet::new();
    let target_head = options
        .basis()
        .commit_identity()
        .map(|identity| identity.commit_id());
    if let Some(head) = target_head {
        parents.push(head);
    }
    for merge_binding in options.merge_parent_bases() {
        let merge_branch = merge_binding.identity().branch_id().clone();
        let result = (|| {
            if !merge_binding.is_current() {
                return Err(crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::InvalidMergeParent {
                        detail: format!("merge parent is foreign or stale: {}", merge_branch.0),
                    },
                ));
            }
            if &merge_branch == branch_id {
                return Ok(());
            }
            let head = merge_binding
                .commit_identity()
                .map(|identity| identity.commit_id())
                .ok_or_else(|| {
                    crate::transactions::data::CommitConflict::new(
                        crate::transactions::data::ConflictClass::InvalidMergeParent {
                            detail: format!("merge parent has no head: {}", merge_branch.0),
                        },
                    )
                })?;
            if !parents.contains(&head) {
                if target_head.is_some() {
                    let inspection = runtime
                        .history
                        .inspect_merge_from_bindings(merge_binding, options.basis())
                        .ok_or_else(|| {
                            crate::transactions::data::CommitConflict::new(
                                crate::transactions::data::ConflictClass::MissingMergeBase {
                                    detail: format!(
                                        "merge parent {} has no common ancestor with {}",
                                        merge_branch.0, branch_id.0
                                    ),
                                },
                            )
                        })?;
                    if !inspection.conflicting_records.is_empty() {
                        return Err(crate::transactions::data::CommitConflict::new(
                            crate::transactions::data::ConflictClass::MergeConflictOverlap {
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
            Ok(())
        })();
        if let Err(conflict) = result {
            emit_history_failure_diagnostic(
                runtime,
                DiagnosticsScope::History,
                conflict.code,
                conflict.detail.clone(),
                merge_parent_resolution_failure_fields(
                    branch_id,
                    &options.merge_parent_branch_ids(),
                ),
            );
            return Err(TransactionCommitError::conflict(conflict));
        }
    }
    Ok((parents, merge_bases.into_iter().collect()))
}

pub(crate) fn resolve_commit_history_for_merge(
    runtime: &RelationalPreparationRuntime,
    options: &RelationalTransactionValidationInput,
    merge_plan: &MergeCommitMutationPlan,
    version_id: VersionId,
    commit_id: CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    resolve_commit_history_for_runtime(
        runtime,
        options,
        Some(merge_plan),
        version_id,
        commit_id,
        |_branch_id| {
            Ok((
                merge_plan.parent_commits.clone_inner(),
                merge_plan.merge_base_commits.iter().copied().collect(),
            ))
        },
    )
}

fn resolve_commit_history_for_runtime<F>(
    runtime: &RelationalPreparationRuntime,
    options: &RelationalTransactionValidationInput,
    merge_plan: Option<&MergeCommitMutationPlan>,
    version_id: VersionId,
    commit_id: CommitId,
    resolve_parents: F,
) -> Result<ResolvedCommitHistory, TransactionCommitError>
where
    F: FnOnce(
        &BranchId,
    )
        -> Result<(Vec<CommitId>, Vec<CommitId>), crate::transactions::data::CommitConflict>,
{
    let branch_id = merge_plan
        .map(|plan| plan.target_branch.clone())
        .unwrap_or_else(|| options.target_branch().clone());
    let requested_merge_parent_count = merge_plan
        .map(|plan| plan.requested_merge_parent_count)
        .unwrap_or(options.merge_parent_bases().len());
    let merge_parent_branches = merge_plan
        .map(|plan| plan.merge_parent_branches.as_ref().to_vec())
        .unwrap_or_else(|| options.merge_parent_branch_ids());
    let branch_basis = options.basis().clone();
    if branch_basis.identity().branch_id() != &branch_id {
        return Err(TransactionCommitError::conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::InvalidMergeParent {
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
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::StaleValidationBasis {
                    detail: "owner-issued branch binding is foreign or no longer current"
                        .to_owned(),
                },
            ),
        ));
    }
    let previous_branch_head_version = branch_basis.commit_identity().map(|head| head.version_id());
    let (parents, merge_base_commits) = match resolve_parents(&branch_id) {
        Ok(result) => result,
        Err(conflict) => {
            emit_history_failure_diagnostic(
                runtime,
                DiagnosticsScope::History,
                conflict.code,
                conflict.detail.clone(),
                merge_parent_resolution_failure_fields(&branch_id, &merge_parent_branches),
            );
            return Err(TransactionCommitError::conflict(conflict));
        }
    };
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

fn emit_history_failure_diagnostic(
    runtime: &RelationalPreparationRuntime,
    scope: DiagnosticsScope,
    code: crate::diagnostics::data::DiagnosticCode,
    message: impl Into<String>,
    fields: impl Into<RelationalDiagnosticFields>,
) {
    runtime.push_bounded_preparation_diagnostic(
        scope,
        DiagnosticsArtifactKind::Failure,
        vec![RelationalDiagnosticsEntry::new(
            code,
            message,
            fields.into(),
        )],
    );
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
