use super::scope::WorthQueryEvidenceScope;

pub(crate) fn graph_application_evidence_scope_as_str(
    scope: WorthQueryEvidenceScope,
) -> &'static str {
    match scope {
        WorthQueryEvidenceScope::GraphCompositionDomainInvariantDenial => {
            "graph-composition-domain-invariant-denial"
        }
        WorthQueryEvidenceScope::GraphCompositionInvariantViolation => {
            "graph-composition-invariant-violation"
        }
        WorthQueryEvidenceScope::GraphTouchDescriptor => "graph-touch-descriptor",
        WorthQueryEvidenceScope::GraphTouchDescriptorRow => "graph-touch-descriptor-row",
        WorthQueryEvidenceScope::ApplicationSupportSectionPosture => {
            "application-support-section-posture"
        }
        WorthQueryEvidenceScope::ApplicationSupportReport => "application-support-report",
        _ => unreachable!("graph/application scope helper called with unrelated scope"),
    }
}
