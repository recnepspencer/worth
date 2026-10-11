mod aspect_value_fields;
mod policy_boundary_fields;
mod record_row_fields;
mod summary_fields;

use crate::diagnostics::data::{
    DeterminismExpectation, DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope,
    RelationalDiagnosticArtifact, RelationalDiagnosticsEntry,
};
use crate::merge::data::{
    ExecutedMergeRecordDiagnosticRow, MergeExecutionDiagnosticsPlan, PreparedMergeExecution,
};
use crate::transactions::data::{MergeExecutionStructuralSummary, MergeExecutionSummary};

use record_row_fields::executed_record_diagnostic_fields;
use summary_fields::{
    merge_execution_artifact_header_fields, merge_execution_failure_fields,
    merge_execution_summary_fields,
};

pub(crate) fn merge_execution_summary_entry(
    summary: &MergeExecutionSummary,
    structural_summary: &MergeExecutionStructuralSummary,
    commit_id: crate::history::data::CommitId,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::MergeExecutionPublished,
        "prepared merge execution published through the authoritative commit lifecycle",
        merge_execution_summary_fields(summary, structural_summary, commit_id),
    )
}

pub(crate) fn merge_execution_success_artifact(
    summary: &MergeExecutionSummary,
    plan: &MergeExecutionDiagnosticsPlan,
    commit_id: crate::history::data::CommitId,
    max_entries: usize,
) -> RelationalDiagnosticArtifact {
    let mut entries = Vec::with_capacity(max_entries.min(plan.executed_records.len() + 1));
    if max_entries > 0 {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::MergeExecutionPublished,
            "prepared merge execution artifact records executed rows only",
            merge_execution_artifact_header_fields(summary, plan, commit_id),
        ));
        entries.extend(
            plan.executed_records
                .iter()
                .take(max_entries.saturating_sub(1))
                .map(executed_record_diagnostics_entry),
        );
    }
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::History,
        DiagnosticsArtifactKind::DetailedTrace,
        DeterminismExpectation::Required,
        entries,
    )
}

pub(crate) fn merge_execution_failure_artifact(
    prepared: &PreparedMergeExecution,
    error: &crate::merge::data::MergeExecutionError,
) -> RelationalDiagnosticArtifact {
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::History,
        DiagnosticsArtifactKind::Failure,
        DeterminismExpectation::Required,
        vec![RelationalDiagnosticsEntry::new(
            merge_execution_failure_code(error),
            "prepared merge execution failed before authoritative merge publication",
            merge_execution_failure_fields(prepared, error),
        )],
    )
}

pub(super) fn emit_merge_execution_failure_artifact(
    runtime: &crate::runtime::RelationalRuntime,
    prepared: &PreparedMergeExecution,
    error: &crate::merge::data::MergeExecutionError,
) {
    runtime
        .publication_authority()
        .push_diagnostic_artifact(merge_execution_failure_artifact(prepared, error));
}

fn executed_record_diagnostics_entry(
    row: &ExecutedMergeRecordDiagnosticRow,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::MergeExecutionPublished,
        "merge execution record row",
        executed_record_diagnostic_fields(row),
    )
}

fn merge_execution_failure_code(error: &crate::merge::data::MergeExecutionError) -> DiagnosticCode {
    match error {
        crate::merge::data::MergeExecutionError::MergeBaseDrift { .. } => {
            DiagnosticCode::MissingMergeBase
        }
        crate::merge::data::MergeExecutionError::Planning(_)
        | crate::merge::data::MergeExecutionError::RuntimeInstanceMismatch { .. }
        | crate::merge::data::MergeExecutionError::StaleBranchHead { .. }
        | crate::merge::data::MergeExecutionError::StaleBranchReference { .. }
        | crate::merge::data::MergeExecutionError::SchemaSemanticDrift { .. }
        | crate::merge::data::MergeExecutionError::Compilation(_)
        | crate::merge::data::MergeExecutionError::MutationPlan(_)
        | crate::merge::data::MergeExecutionError::Commit(_) => {
            DiagnosticCode::DeterministicMergeViolation
        }
    }
}
