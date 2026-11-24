use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use contracts::history::data::commit_receipt::OrderedParentList;
use contracts::history::data::coordinates::BranchId;
use contracts::history::data::coordinates::CommitId;
use contracts::history::data::history_policy::HistoryShapeClassification;
use contracts::snapshots::data::coordinates::SnapshotId;
use worth_relational_contracts::internal_port as contracts;

use super::post_commit_consumer::PostCommitConsumptionFailure;

pub(super) fn build_publication_diagnostic_entries(
    commit_id: CommitId,
    snapshot_id: SnapshotId,
    branch_id: &BranchId,
    parents: &[CommitId],
    merge_parent_branches: &[BranchId],
    merge_base_commits: &[CommitId],
    consumer_failure: Option<PostCommitConsumptionFailure>,
) -> Vec<RelationalDiagnosticsEntry> {
    let authoritative_parent_list = OrderedParentList::from_authoritative(parents.to_vec());
    let history_shape = authoritative_parent_list.history_shape_classification();
    let publication_code = if history_shape == HistoryShapeClassification::MergeReady {
        DiagnosticCode::MergeCommitPublished
    } else {
        DiagnosticCode::CommitPublished
    };
    let mut entries = Vec::new();
    if history_shape == HistoryShapeClassification::MergeReady {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::MergeBaseResolved,
            "ancestry-derived merge-base result resolved deterministically",
            merge_base_resolved_fields(
                commit_id,
                history_shape,
                authoritative_parent_list.as_slice(),
                merge_base_commits,
            ),
        ));
    }
    entries.push(RelationalDiagnosticsEntry::new(
        publication_code,
        if history_shape == HistoryShapeClassification::MergeReady {
            "merge-ready history commit published coherently"
        } else {
            "commit published coherently"
        },
        publication_fields(
            commit_id,
            snapshot_id,
            branch_id,
            history_shape,
            authoritative_parent_list.as_slice(),
            merge_parent_branches,
            merge_base_commits,
        ),
    ));
    if matches!(
        consumer_failure,
        Some(PostCommitConsumptionFailure::ConsumerFailureNonAuthoritative)
    ) {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::PreparationFailure,
            "post-commit consumer failed without affecting publication",
            post_commit_consumer_failure_fields(commit_id, snapshot_id),
        ));
    }
    entries
}

fn merge_base_resolved_fields(
    commit_id: CommitId,
    history_shape: HistoryShapeClassification,
    authoritative_parent_list: &[CommitId],
    merge_base_commits: &[CommitId],
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        ("commit_id", RelationalDiagnosticValue::CommitId(commit_id)),
        ("history_shape", history_shape_value(history_shape)),
        (
            "authoritative_parent_list",
            commit_id_array(authoritative_parent_list),
        ),
        ("merge_base_commit_ids", commit_id_array(merge_base_commits)),
    ])
    .into()
}

fn publication_fields(
    commit_id: CommitId,
    snapshot_id: SnapshotId,
    branch_id: &BranchId,
    history_shape: HistoryShapeClassification,
    authoritative_parent_list: &[CommitId],
    merge_parent_branches: &[BranchId],
    merge_base_commits: &[CommitId],
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        ("commit_id", RelationalDiagnosticValue::CommitId(commit_id)),
        (
            "snapshot_id",
            RelationalDiagnosticValue::SnapshotId(snapshot_id),
        ),
        (
            "branch_id",
            RelationalDiagnosticValue::BranchId(branch_id.clone()),
        ),
        ("history_shape", history_shape_value(history_shape)),
        (
            "parent_count",
            RelationalDiagnosticValue::unsigned(authoritative_parent_list.len()),
        ),
        (
            "authoritative_parent_list",
            commit_id_array(authoritative_parent_list),
        ),
        (
            "merge_parent_branches",
            branch_id_array(merge_parent_branches),
        ),
        ("merge_base_commit_ids", commit_id_array(merge_base_commits)),
    ])
    .into()
}

fn post_commit_consumer_failure_fields(
    commit_id: CommitId,
    snapshot_id: SnapshotId,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "failure_class",
            RelationalDiagnosticValue::string("consumer_failure_non_authoritative"),
        ),
        ("commit_id", RelationalDiagnosticValue::CommitId(commit_id)),
        (
            "snapshot_id",
            RelationalDiagnosticValue::SnapshotId(snapshot_id),
        ),
    ])
    .into()
}

fn history_shape_value(history_shape: HistoryShapeClassification) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::string(format!("{history_shape:?}"))
}

fn commit_id_array(commit_ids: &[CommitId]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        commit_ids
            .iter()
            .copied()
            .map(RelationalDiagnosticValue::CommitId),
    )
}

fn branch_id_array(branch_ids: &[BranchId]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        branch_ids
            .iter()
            .cloned()
            .map(RelationalDiagnosticValue::BranchId),
    )
}
