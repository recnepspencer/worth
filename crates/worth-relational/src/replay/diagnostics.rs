use crate::diagnostics::data::{DiagnosticCode, DiagnosticsScope, RelationalDiagnosticValue};
use crate::replay::data::{
    RelationalReplayOutcome, RelationalReplayRequest, ReplayFailureClass,
    ReplayLineageAuthorityBasis, ReplayMismatch,
};
use crate::runtime::RelationalRuntime;

pub(super) fn record_replay_diagnostic(
    runtime: &RelationalRuntime,
    request: &RelationalReplayRequest,
    outcome: &RelationalReplayOutcome,
) {
    let code = match outcome.failure.as_ref() {
        Some(ReplayFailureClass::SchemaMismatch) => DiagnosticCode::ReplaySchemaVersionMismatch,
        Some(_) => DiagnosticCode::InvariantViolation,
        None => DiagnosticCode::CommitPublished,
    };
    let builder = runtime
        .publication_authority()
        .diagnostic(DiagnosticsScope::Replay);
    let builder = if outcome.failure.is_some() {
        builder.failure()
    } else {
        builder.comparison()
    };
    builder.emit_entry(
        code,
        "replay comparison completed",
        replay_comparison_fields(request, outcome),
    );
}

fn replay_comparison_fields(
    request: &RelationalReplayRequest,
    outcome: &RelationalReplayOutcome,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "commit_id",
            RelationalDiagnosticValue::CommitId(request.commit_id),
        ),
        (
            "branch_id",
            RelationalDiagnosticValue::BranchId(request.branch_id.clone()),
        ),
        (
            "verification_mode",
            RelationalDiagnosticValue::string(format!("{:?}", request.verification_mode)),
        ),
        (
            "lineage_authority_basis",
            lineage_authority_basis_kind(outcome.lineage_authority_basis.as_ref()),
        ),
        (
            "lineage_authority_commit_id",
            lineage_authority_commit_id(outcome.lineage_authority_basis.as_ref()),
        ),
        (
            "lineage_authority_event_count",
            lineage_authority_event_count(outcome.lineage_authority_basis.as_ref()),
        ),
        (
            "lineage_authority_decision_count",
            lineage_authority_decision_count(outcome.lineage_authority_basis.as_ref()),
        ),
        (
            "compared_surfaces",
            RelationalDiagnosticValue::array(
                outcome
                    .compared_surfaces
                    .iter()
                    .map(|surface| RelationalDiagnosticValue::string(format!("{surface:?}"))),
            ),
        ),
        (
            "mismatch_count",
            RelationalDiagnosticValue::unsigned(outcome.mismatches.len()),
        ),
        ("mismatch_classes", mismatch_classes(&outcome.mismatches)),
        ("mismatch_surfaces", mismatch_surfaces(&outcome.mismatches)),
        (
            "mismatch_verification_layers",
            mismatch_verification_layers(&outcome.mismatches),
        ),
        (
            "failure",
            RelationalDiagnosticValue::optional(
                outcome
                    .failure
                    .as_ref()
                    .map(|failure| RelationalDiagnosticValue::string(format!("{failure:?}"))),
            ),
        ),
    ])
}

fn lineage_authority_basis_kind(
    basis: Option<&ReplayLineageAuthorityBasis>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(
        basis.map(|basis| RelationalDiagnosticValue::string(format!("{:?}", basis.kind()))),
    )
}

fn lineage_authority_commit_id(
    basis: Option<&ReplayLineageAuthorityBasis>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(
        basis.map(|basis| RelationalDiagnosticValue::CommitId(basis.commit_id())),
    )
}

fn lineage_authority_event_count(
    basis: Option<&ReplayLineageAuthorityBasis>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(
        basis.map(|basis| RelationalDiagnosticValue::unsigned(basis.lineage_event_count())),
    )
}

fn lineage_authority_decision_count(
    basis: Option<&ReplayLineageAuthorityBasis>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(
        basis.map(|basis| RelationalDiagnosticValue::unsigned(basis.lineage_decision_count())),
    )
}

fn mismatch_classes(mismatches: &[ReplayMismatch]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        mismatches
            .iter()
            .map(|mismatch| RelationalDiagnosticValue::string(format!("{:?}", mismatch.class))),
    )
}

fn mismatch_surfaces(mismatches: &[ReplayMismatch]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        mismatches
            .iter()
            .map(|mismatch| RelationalDiagnosticValue::string(format!("{:?}", mismatch.surface))),
    )
}

fn mismatch_verification_layers(mismatches: &[ReplayMismatch]) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(mismatches.iter().map(|mismatch| {
        RelationalDiagnosticValue::string(format!("{:?}", mismatch.verification_layer))
    }))
}
