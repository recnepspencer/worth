use crate::diagnostics::data::{DiagnosticCode, DiagnosticsScope};
use crate::replay::data::{RelationalReplayOutcome, RelationalReplayRequest, ReplayFailureClass};
use crate::runtime::RelationalRuntime;

use crate::replay::diagnostics::replay_comparison_fields;

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
