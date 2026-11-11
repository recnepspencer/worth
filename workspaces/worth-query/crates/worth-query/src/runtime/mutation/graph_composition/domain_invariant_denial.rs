use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::runtime::{
    WorthQueryGraphCompositionAdmissionTrace, WorthQueryGraphCompositionAdmissionTraceStage,
    WorthQueryGraphCompositionDomainInvariantSummary,
};

const CONTRIBUTED_INVARIANT_OWNER_FAMILY: &str = "domain_capability_invariant_owner";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionDomainInvariantDenial {
    owner_family: String,
    invariant_family: String,
    message: String,
    domain_invariant_summary: WorthQueryGraphCompositionDomainInvariantSummary,
    admission_trace: WorthQueryGraphCompositionAdmissionTrace,
    denial_digest: String,
}

impl WorthQueryGraphCompositionDomainInvariantDenial {
    pub(crate) fn from_contributed(
        invariant_family: impl Into<String>,
        message: impl Into<String>,
        domain_invariant_summary: WorthQueryGraphCompositionDomainInvariantSummary,
    ) -> Result<Self, IdentityDerivationDenial> {
        let invariant_family = invariant_family.into();
        let message = message.into();
        let violation_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::GraphCompositionInvariantViolation,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("invariant_family"),
            invariant_family.as_str(),
        )
        .seal()?;
        Self::build(
            invariant_family,
            message,
            domain_invariant_summary,
            violation_digest,
        )
    }

    pub fn owner_family(&self) -> &str {
        &self.owner_family
    }

    pub fn invariant_family(&self) -> &str {
        &self.invariant_family
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn domain_invariant_summary(&self) -> &WorthQueryGraphCompositionDomainInvariantSummary {
        &self.domain_invariant_summary
    }

    pub fn admission_trace(&self) -> &WorthQueryGraphCompositionAdmissionTrace {
        &self.admission_trace
    }

    pub fn failure_stage(&self) -> WorthQueryGraphCompositionAdmissionTraceStage {
        self.admission_trace.failure_stage()
    }

    pub fn denial_digest(&self) -> &str {
        &self.denial_digest
    }
}

impl WorthQueryGraphCompositionDomainInvariantDenial {
    fn build(
        invariant_family: String,
        message: String,
        domain_invariant_summary: WorthQueryGraphCompositionDomainInvariantSummary,
        violation_digest: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        use WorthQueryGraphCompositionAdmissionTraceStage as Stage;

        let admission_trace = WorthQueryGraphCompositionAdmissionTrace::new(
            vec![
                Stage::ProgramParsed,
                Stage::SymbolsValidated,
                Stage::LoweringValidated,
                Stage::DomainInvariantEvaluated,
                Stage::DeniedBeforeExecution,
            ],
            Stage::DomainInvariantEvaluated,
        )?;
        let denial_digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::GraphCompositionDomainInvariantDenial,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("owner_family"),
            CONTRIBUTED_INVARIANT_OWNER_FAMILY,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("invariant_family"),
            invariant_family.as_str(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("summary_digest"),
            domain_invariant_summary.summary_evidence_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("admission_trace_digest"),
            admission_trace.admission_trace_evidence_digest(),
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("violation_digest"),
            &violation_digest,
        )
        .seal()?
        .as_str()
        .to_string();
        Ok(Self {
            owner_family: CONTRIBUTED_INVARIANT_OWNER_FAMILY.to_string(),
            invariant_family,
            message,
            domain_invariant_summary,
            admission_trace,
            denial_digest,
        })
    }
}

impl std::fmt::Display for WorthQueryGraphCompositionDomainInvariantDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "graph composition denied by {} for invariant `{}`: {}",
            self.owner_family, self.invariant_family, self.message
        )
    }
}
