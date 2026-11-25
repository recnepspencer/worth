use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use mutation::validation::engine::result::InvariantExecutionResult;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

use mutation::validation::invariant_authority::diagnostic_projection::custom_trace_diagnostic_fields;
use mutation::validation::invariant_authority::diagnostic_projection::preparation_failure_diagnostic_fields;
use mutation::validation::invariant_authority::diagnostic_projection::proof_boundary_trace_diagnostic_fields;
use mutation::validation::invariant_authority::diagnostic_projection::serial_preparation_diagnostic_fields;

pub fn emit_preparation_diagnostics(
    diagnostics: PreparationDiagnostics<'_>,
    result: &InvariantExecutionResult,
) {
    if diagnostics.profile.should_capture_artifact(
        DiagnosticsScope::Invariant,
        DiagnosticsArtifactKind::DetailedTrace,
    ) {
        if let Some(proof_boundary) = result.proof_boundary_artifact() {
            diagnostics.emit(
                DiagnosticsScope::Invariant,
                DiagnosticsArtifactKind::DetailedTrace,
                vec![RelationalDiagnosticsEntry::new(
                    DiagnosticCode::InvariantProofBoundaryObserved,
                    "invariant execution preserved an explicit planner/executor proof boundary",
                    proof_boundary_trace_diagnostic_fields(
                        result.metadata().execution_point(),
                        &proof_boundary,
                    ),
                )],
            );
        }
        let custom_trace_entries = result
            .results()
            .iter()
            .filter_map(custom_invariant_trace_entry)
            .collect::<Vec<_>>();
        if !custom_trace_entries.is_empty() {
            diagnostics.emit(
                DiagnosticsScope::Invariant,
                DiagnosticsArtifactKind::DetailedTrace,
                custom_trace_entries,
            );
        }
    }
    let serial_selection_reason = result
        .metadata()
        .preparation_strategy()
        .and_then(|strategy| strategy.serial_selection_reason);
    let failures = result.metadata().preparation_failures();
    if serial_selection_reason.is_none() && failures.is_empty() {
        return;
    }

    let mut entries = Vec::new();
    if let Some(reason) = serial_selection_reason {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::SerialPreparationSelected,
            "preparation selected serial execution",
            serial_preparation_diagnostic_fields(result.metadata().execution_point(), reason),
        ));
    }
    for failure in failures {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::PreparationFailure,
            "preparation contract reported a structured failure",
            preparation_failure_diagnostic_fields(result.metadata().execution_point(), *failure),
        ));
    }

    diagnostics.emit(
        DiagnosticsScope::Invariant,
        DiagnosticsArtifactKind::DetailedTrace,
        entries,
    );
}

fn custom_invariant_trace_entry(
    result: &mutation::validation::engine::result::check_result::InvariantCheckResult,
) -> Option<RelationalDiagnosticsEntry> {
    let artifact =
        mutation::validation::engine::result::InvariantExecutionResult::custom_trace_artifact(
            result,
        )?;
    Some(RelationalDiagnosticsEntry::new(
        DiagnosticCode::InvariantProofBoundaryObserved,
        "custom invariant structural provenance captured for deterministic debugging",
        custom_trace_diagnostic_fields(&artifact),
    ))
}
