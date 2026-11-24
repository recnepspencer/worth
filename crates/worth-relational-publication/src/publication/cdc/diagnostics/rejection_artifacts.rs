use crate::publication::cdc::data::SubscriberStreamFailureClass;
use contracts::diagnostics::data::artifacts::DeterminismExpectation;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::diagnostics::data::fields::RelationalDiagnosticFields;
use contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use worth_relational_contracts::internal_port as contracts;

pub fn rejection_artifact(
    class: SubscriberStreamFailureClass,
    detail: &str,
) -> RelationalDiagnosticArtifact {
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Replay,
        DiagnosticsArtifactKind::Failure,
        DeterminismExpectation::Required,
        vec![RelationalDiagnosticsEntry::new(
            DiagnosticCode::ReplaySchemaVersionMismatch,
            "subscriber recovery request rejected",
            subscriber_recovery_request_rejection_fields(class, detail),
        )],
    )
}

fn subscriber_recovery_request_rejection_fields(
    failure_class: SubscriberStreamFailureClass,
    detail: &str,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "failure_class",
            RelationalDiagnosticValue::string(format!("{failure_class:?}")),
        ),
        ("detail", RelationalDiagnosticValue::string(detail)),
    ])
    .into()
}
