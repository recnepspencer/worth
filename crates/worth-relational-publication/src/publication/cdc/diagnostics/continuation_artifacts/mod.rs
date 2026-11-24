use worth_relational_contracts::internal_port as contracts;
pub mod boundary_fields;
pub mod contract_fields;
pub mod diagnostic_terms;

use crate::publication::cdc::data::SubscriberBoundaryAssessment;
use crate::publication::cdc::data::SubscriberContinuationAssessment;
use crate::publication::cdc::data::SubscriberStreamFailureClass;
use contracts::diagnostics::data::artifacts::DeterminismExpectation;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::publication::cdc::data::subscriber_contract::SubscriberContractDeclaration;
use contracts::schema::data::continuity::transition_classification::SchemaContinuationClassification;

use boundary_fields::{boundary_rejection_fields, boundary_summary_fields};
use contract_fields::{
    continuation_rejection_fields, continuation_rejection_renegotiation_fields,
    continuation_rejection_upgrade_fields, continuation_renegotiation_fields,
    continuation_summary_fields, continuation_upgrade_fields,
};

pub(crate) fn continuation_summary_artifact(
    assessment: &SubscriberContinuationAssessment,
    contract_id: &str,
) -> RelationalDiagnosticArtifact {
    let mut entries = vec![RelationalDiagnosticsEntry::new(
        DiagnosticCode::SubscriberContractEvaluated,
        "subscriber continuation assessment completed",
        continuation_summary_fields(assessment, contract_id),
    )];
    if assessment.contract_upgrade_applied() {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::SubscriberContractUpgradeDecision,
            "subscriber continuation applied declared contract upgrade support",
            continuation_upgrade_fields(assessment, contract_id),
        ));
    }
    if assessment.continuation_outcome() == SchemaContinuationClassification::RequireRenegotiation {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::SubscriberRenegotiationDecision,
            "subscriber continuation requires explicit renegotiation",
            continuation_renegotiation_fields(assessment, contract_id),
        ));
    }
    entries.extend(
        assessment
            .boundary_assessments()
            .iter()
            .map(boundary_summary_entry),
    );
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Replay,
        DiagnosticsArtifactKind::MinimalSummary,
        DeterminismExpectation::Required,
        entries,
    )
}

pub fn continuation_rejection_artifact(
    assessment: &SubscriberContinuationAssessment,
    class: SubscriberStreamFailureClass,
    detail: &str,
    subscriber_contract: &SubscriberContractDeclaration,
    normalized_boundary_count_at_failure: usize,
) -> RelationalDiagnosticArtifact {
    let mut entries = vec![RelationalDiagnosticsEntry::new(
        DiagnosticCode::SubscriberContractEvaluated,
        "subscriber continuation assessment rejected",
        continuation_rejection_fields(
            class,
            detail,
            subscriber_contract,
            normalized_boundary_count_at_failure,
        ),
    )];
    if class == SubscriberStreamFailureClass::ContractUpgradeUnsupported {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::SubscriberContractUpgradeDecision,
            "subscriber continuation rejected because contract upgrade support was not declared",
            continuation_rejection_upgrade_fields(
                subscriber_contract,
                normalized_boundary_count_at_failure,
            ),
        ));
    }
    if class == SubscriberStreamFailureClass::RenegotiationRequired {
        entries.push(RelationalDiagnosticsEntry::new(
            DiagnosticCode::SubscriberRenegotiationDecision,
            "subscriber continuation rejected because renegotiation is required",
            continuation_rejection_renegotiation_fields(
                subscriber_contract,
                normalized_boundary_count_at_failure,
            ),
        ));
    }
    entries.extend(
        assessment
            .boundary_assessments()
            .iter()
            .map(|boundary| boundary_rejection_entry(boundary, subscriber_contract)),
    );
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Replay,
        DiagnosticsArtifactKind::Failure,
        DeterminismExpectation::Required,
        entries,
    )
}

fn boundary_summary_entry(boundary: &SubscriberBoundaryAssessment) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SubscriberBoundaryEvaluated,
        "subscriber boundary assessed against declared contract",
        boundary_summary_fields(boundary),
    )
}

fn boundary_rejection_entry(
    boundary: &SubscriberBoundaryAssessment,
    subscriber_contract: &SubscriberContractDeclaration,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SubscriberBoundaryEvaluated,
        "subscriber boundary rejected against declared contract",
        boundary_rejection_fields(boundary, subscriber_contract),
    )
}
