use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_foundational::{
    foundational_diagnostic_boundary_artifact_subject, foundational_diagnostic_code,
    foundational_diagnostic_locator_boundary_artifact, foundational_diagnostic_scope,
    BoundaryArtifactField, BoundaryArtifactLocator, FoundationalDiagnosticDenialClass,
    FoundationalDiagnosticEvidencePosture, FoundationalDiagnosticLocalityClaim,
    FoundationalDiagnosticOutcomeKind, FoundationalDiagnosticProvenanceReadyRow,
    FoundationalDiagnosticRow, FoundationalDiagnosticSemanticLabelSet,
    FoundationalDiagnosticSeverity, FoundationalDiagnosticSupportEvidencePosture,
    FoundationalDiagnosticSupportRow, FoundationalDiagnosticWidenedFalloutPosture,
};

use super::identity::{
    boundary_artifact_id, diagnostic_code_identity, diagnostic_label_identity,
    diagnostic_scope_identity,
};

use super::super::payloads::{
    WorthQueryDomainCapabilityPayload, WorthQueryDomainCapabilitySemanticPosture,
};
use super::super::targets::WorthQueryDomainCapabilityTargetBinding;
use super::super::{
    WorthQueryDomainCapabilityTargetKind,
    WorthQueryMaterializationReadyDomainCapabilityContribution,
};

pub(crate) struct WorthQueryDomainCapabilityDiagnosticRows {
    pub subject: worth_foundational::FoundationalDiagnosticSubject,
    pub scope: worth_foundational::FoundationalDiagnosticScopeId,
    pub primary_code: worth_foundational::FoundationalDiagnosticCodeId,
    pub outcome_kind: FoundationalDiagnosticOutcomeKind,
    pub required_rows: Vec<FoundationalDiagnosticRow>,
    pub standard_rows: Vec<FoundationalDiagnosticRow>,
    pub forensic_rows: Vec<FoundationalDiagnosticRow>,
}

pub(crate) fn build_rows<P, T>(
    contribution: &WorthQueryMaterializationReadyDomainCapabilityContribution<P, T>,
) -> Result<WorthQueryDomainCapabilityDiagnosticRows, IdentityDerivationDenial>
where
    P: WorthQueryDomainCapabilityPayload,
    T: WorthQueryDomainCapabilityTargetBinding,
{
    let payload = contribution.payload().payload();
    let category = payload.category();
    let semantic_posture = payload.semantic_posture();
    let target = contribution.payload().target();
    let artifact_id = boundary_artifact_id(contribution.payload().request_identity());
    let subject = foundational_diagnostic_boundary_artifact_subject(
        artifact_id,
        BoundaryArtifactField::Payload,
    );
    let locator = foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
        artifact_id,
        BoundaryArtifactField::Payload,
    ));
    let scope = scope_id(diagnostic_scope_identity(category, target.kind())?.as_str());
    let primary_code = code_id(payload.semantic_code());
    let outcome_kind = semantic_posture.outcome_kind();
    let labels = diagnostic_labels(category, target.kind(), outcome_kind)?;
    let evidence_posture = evidence_posture_for(target.kind(), semantic_posture, outcome_kind);
    let support_evidence_posture =
        support_evidence_posture_for(target.kind(), semantic_posture, outcome_kind);
    let locality = locality_for(target.kind());
    let widened = widened_for(outcome_kind);

    let required_rows = vec![
        FoundationalDiagnosticRow::Decision(
            worth_foundational::FoundationalDiagnosticDecisionRow::new(
                primary_code.clone(),
                scope.clone(),
                severity_for(outcome_kind),
                subject.clone(),
                locator.clone(),
                outcome_kind,
                labels.clone(),
                denial_class_for(outcome_kind),
                locality,
                widened,
            ),
        ),
        FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
            diagnostic_support_code(payload.semantic_code())?,
            scope.clone(),
            severity_for_support(outcome_kind),
            subject.clone(),
            locator.clone(),
            outcome_kind,
            labels.clone(),
            support_evidence_posture.clone(),
            locality,
            widened,
        )),
    ];
    let standard_rows = vec![FoundationalDiagnosticRow::ProvenanceReady(
        FoundationalDiagnosticProvenanceReadyRow::new(
            diagnostic_provenance_code(payload.semantic_code())?,
            scope.clone(),
            FoundationalDiagnosticSeverity::Info,
            subject.clone(),
            locator.clone(),
            outcome_kind,
            labels.clone(),
            foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
                boundary_artifact_id(&target.binding_identity()),
                BoundaryArtifactField::Basis,
            )),
            evidence_posture,
        ),
    )];
    let forensic_rows = vec![FoundationalDiagnosticRow::Support(
        FoundationalDiagnosticSupportRow::new(
            diagnostic_trace_code(payload.semantic_code())?,
            scope.clone(),
            FoundationalDiagnosticSeverity::Info,
            subject.clone(),
            locator.clone(),
            outcome_kind,
            labels,
            support_evidence_posture,
            locality,
            widened,
        ),
    )];

    Ok(WorthQueryDomainCapabilityDiagnosticRows {
        subject,
        scope,
        primary_code,
        outcome_kind,
        required_rows,
        standard_rows,
        forensic_rows,
    })
}

fn severity_for(outcome: FoundationalDiagnosticOutcomeKind) -> FoundationalDiagnosticSeverity {
    match outcome {
        FoundationalDiagnosticOutcomeKind::Accepted => FoundationalDiagnosticSeverity::Info,
        FoundationalDiagnosticOutcomeKind::Advisory
        | FoundationalDiagnosticOutcomeKind::Deferred
        | FoundationalDiagnosticOutcomeKind::Partial => FoundationalDiagnosticSeverity::Advisory,
        FoundationalDiagnosticOutcomeKind::Unsupported
        | FoundationalDiagnosticOutcomeKind::Denied => FoundationalDiagnosticSeverity::Denial,
        FoundationalDiagnosticOutcomeKind::Mismatch => FoundationalDiagnosticSeverity::Warning,
        FoundationalDiagnosticOutcomeKind::Violation => FoundationalDiagnosticSeverity::Violation,
    }
}

fn severity_for_support(
    outcome: FoundationalDiagnosticOutcomeKind,
) -> FoundationalDiagnosticSeverity {
    match outcome {
        FoundationalDiagnosticOutcomeKind::Violation
        | FoundationalDiagnosticOutcomeKind::Denied
        | FoundationalDiagnosticOutcomeKind::Unsupported => {
            FoundationalDiagnosticSeverity::Advisory
        }
        _ => FoundationalDiagnosticSeverity::Info,
    }
}

fn denial_class_for(
    outcome: FoundationalDiagnosticOutcomeKind,
) -> Option<FoundationalDiagnosticDenialClass> {
    match outcome {
        FoundationalDiagnosticOutcomeKind::Denied => {
            Some(FoundationalDiagnosticDenialClass::DomainDenied)
        }
        FoundationalDiagnosticOutcomeKind::Unsupported => {
            Some(FoundationalDiagnosticDenialClass::UnsupportedDenied)
        }
        FoundationalDiagnosticOutcomeKind::Violation => {
            Some(FoundationalDiagnosticDenialClass::PolicyDenied)
        }
        _ => None,
    }
}

fn evidence_posture_for(
    _target_kind: WorthQueryDomainCapabilityTargetKind,
    semantic_posture: WorthQueryDomainCapabilitySemanticPosture,
    outcome: FoundationalDiagnosticOutcomeKind,
) -> FoundationalDiagnosticEvidencePosture {
    if semantic_posture.is_policy_or_inferred() {
        return FoundationalDiagnosticEvidencePosture::Summarized;
    }
    match outcome {
        FoundationalDiagnosticOutcomeKind::Partial
        | FoundationalDiagnosticOutcomeKind::Advisory => {
            FoundationalDiagnosticEvidencePosture::Summarized
        }
        _ => FoundationalDiagnosticEvidencePosture::RetainedDirect,
    }
}

fn support_evidence_posture_for(
    target_kind: WorthQueryDomainCapabilityTargetKind,
    semantic_posture: WorthQueryDomainCapabilitySemanticPosture,
    outcome: FoundationalDiagnosticOutcomeKind,
) -> FoundationalDiagnosticSupportEvidencePosture {
    FoundationalDiagnosticSupportEvidencePosture::Present(evidence_posture_for(
        target_kind,
        semantic_posture,
        outcome,
    ))
}

fn locality_for(
    target_kind: WorthQueryDomainCapabilityTargetKind,
) -> FoundationalDiagnosticLocalityClaim {
    match target_kind {
        WorthQueryDomainCapabilityTargetKind::IntentDeclaration
        | WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan => {
            FoundationalDiagnosticLocalityClaim::ExactSubject
        }
        WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope => {
            FoundationalDiagnosticLocalityClaim::SubjectNeighborhood
        }
    }
}

fn widened_for(
    outcome: FoundationalDiagnosticOutcomeKind,
) -> FoundationalDiagnosticWidenedFalloutPosture {
    match outcome {
        FoundationalDiagnosticOutcomeKind::Partial
        | FoundationalDiagnosticOutcomeKind::Advisory => {
            FoundationalDiagnosticWidenedFalloutPosture::WidenedExpected
        }
        _ => FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    }
}

fn diagnostic_labels(
    category: super::super::payloads::WorthQueryDomainCapabilityCategory,
    target_kind: WorthQueryDomainCapabilityTargetKind,
    outcome_kind: FoundationalDiagnosticOutcomeKind,
) -> Result<FoundationalDiagnosticSemanticLabelSet, IdentityDerivationDenial> {
    Ok(FoundationalDiagnosticSemanticLabelSet::new([
        diagnostic_label_code("category", category.as_str())?,
        diagnostic_label_code("target", target_kind.as_str())?,
        diagnostic_label_code("outcome", outcome_kind.canonical_name())?,
    ]))
}

fn diagnostic_label_code(
    role: &'static str,
    value: &str,
) -> Result<worth_foundational::FoundationalDiagnosticCodeId, IdentityDerivationDenial> {
    Ok(code_id(diagnostic_label_identity(role, value)?.as_str()))
}

fn diagnostic_support_code(
    semantic_code: &str,
) -> Result<worth_foundational::FoundationalDiagnosticCodeId, IdentityDerivationDenial> {
    Ok(code_id(
        diagnostic_code_identity("support", semantic_code)?.as_str(),
    ))
}

fn diagnostic_provenance_code(
    semantic_code: &str,
) -> Result<worth_foundational::FoundationalDiagnosticCodeId, IdentityDerivationDenial> {
    Ok(code_id(
        diagnostic_code_identity("provenance", semantic_code)?.as_str(),
    ))
}

fn diagnostic_trace_code(
    semantic_code: &str,
) -> Result<worth_foundational::FoundationalDiagnosticCodeId, IdentityDerivationDenial> {
    Ok(code_id(
        diagnostic_code_identity("trace", semantic_code)?.as_str(),
    ))
}

fn code_id(value: &str) -> worth_foundational::FoundationalDiagnosticCodeId {
    foundational_diagnostic_code(normalize_fragment(value)).expect("normalized diagnostic code")
}

fn scope_id(value: &str) -> worth_foundational::FoundationalDiagnosticScopeId {
    foundational_diagnostic_scope(normalize_fragment(value)).expect("normalized diagnostic scope")
}

fn normalize_fragment(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut previous_separator = true;
    for ch in value.chars().flat_map(char::to_lowercase) {
        let valid = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        if valid {
            normalized.push(ch);
            previous_separator = false;
        } else if !previous_separator {
            normalized.push('.');
            previous_separator = true;
        }
    }
    while normalized.ends_with('.') {
        normalized.pop();
    }
    if normalized.is_empty() {
        "unspecified".to_string()
    } else {
        normalized
    }
}
