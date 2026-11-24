use crate::contracts::diagnostics::data::artifacts::{
    DiagnosticCode, DiagnosticsArtifactKind, DiagnosticsScope, RelationalDiagnosticsEntry,
};
use crate::contracts::diagnostics::data::fields::{
    RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::indexes::data::definition::DerivedIndexId;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;

pub fn derived_index_build_artifact_kind(
    failed_indexes: &[DerivedIndexId],
) -> DiagnosticsArtifactKind {
    if failed_indexes.is_empty() {
        DiagnosticsArtifactKind::MinimalSummary
    } else {
        DiagnosticsArtifactKind::Failure
    }
}

pub fn derived_index_build_completed(
    source_commit_id: CommitId,
    branch_id: &BranchId,
    generations: &[DerivedIndexGeneration],
    failed_indexes: &[DerivedIndexId],
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        derived_index_build_diagnostic_code(failed_indexes),
        "derived index build completed",
        derived_index_build_completed_fields(
            source_commit_id,
            branch_id,
            generations,
            failed_indexes,
        ),
    )
}

pub fn derived_index_build_scope() -> DiagnosticsScope {
    DiagnosticsScope::QueryPlanning
}

fn derived_index_build_completed_fields(
    source_commit_id: CommitId,
    branch_id: &BranchId,
    generations: &[DerivedIndexGeneration],
    failed_indexes: &[DerivedIndexId],
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "source_commit_id",
            RelationalDiagnosticValue::CommitId(source_commit_id),
        ),
        (
            "branch_id",
            RelationalDiagnosticValue::BranchId(branch_id.clone()),
        ),
        (
            "generation_count",
            RelationalDiagnosticValue::unsigned(generations.len()),
        ),
        (
            "generation_ids",
            RelationalDiagnosticValue::array(generations.iter().map(|generation| {
                RelationalDiagnosticValue::DerivedIndexGenerationId(generation.generation_id)
            })),
        ),
        (
            "failed_indexes",
            RelationalDiagnosticValue::array(
                failed_indexes
                    .iter()
                    .copied()
                    .map(RelationalDiagnosticValue::DerivedIndexId),
            ),
        ),
    ])
    .into()
}

fn derived_index_build_diagnostic_code(failed_indexes: &[DerivedIndexId]) -> DiagnosticCode {
    if failed_indexes.is_empty() {
        DiagnosticCode::CommitPublished
    } else {
        DiagnosticCode::DiagnosticsPublicationFailure
    }
}
