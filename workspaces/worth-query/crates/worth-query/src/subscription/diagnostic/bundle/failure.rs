use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::admission_error::QuerySubscriptionAdmissionError;
use super::super::super::bridge_lowering_error::QuerySubscriptionBridgeLoweringError;
use super::super::super::certification::SubscriptionLifecycleCertificationError;
use super::super::super::declaration_error::QuerySubscriptionDeclarationDenial;
use super::super::super::error::QuerySubscriptionFamilySelectionError;
use super::super::super::evidence_identities::diagnostic_failure_identity;
use super::super::super::support::QuerySubscriptionSupportReportError;
use super::super::stage::{QuerySubscriptionDiagnosticOutcome, QuerySubscriptionDiagnosticStage};
use super::evidence::QuerySubscriptionDiagnosticCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionDiagnosticBundleErrorKind {
    MissingRequiredStage,
    SelectionContextMismatch,
    DeclarationSourceMismatch,
    BridgeLoweringSourceMismatch,
    AdmissionSourceMismatch,
    SupportSourceMismatch,
    LifecycleSourceMismatch,
    FailureSourceMismatch,
}

impl QuerySubscriptionDiagnosticBundleErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingRequiredStage => "missing_required_stage",
            Self::SelectionContextMismatch => "selection_context_mismatch",
            Self::DeclarationSourceMismatch => "declaration_source_mismatch",
            Self::BridgeLoweringSourceMismatch => "bridge_lowering_source_mismatch",
            Self::AdmissionSourceMismatch => "admission_source_mismatch",
            Self::SupportSourceMismatch => "support_source_mismatch",
            Self::LifecycleSourceMismatch => "lifecycle_source_mismatch",
            Self::FailureSourceMismatch => "failure_source_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticBundleError {
    error_kind: QuerySubscriptionDiagnosticBundleErrorKind,
    message: &'static str,
    failure_identity: WorthQueryEvidenceIdentity,
    counters: QuerySubscriptionDiagnosticCounters,
}

impl QuerySubscriptionDiagnosticBundleError {
    pub(crate) fn new(
        error_kind: QuerySubscriptionDiagnosticBundleErrorKind,
        message: &'static str,
        evidence_parts: &[String],
    ) -> Result<Self, IdentityDerivationDenial> {
        let counters = QuerySubscriptionDiagnosticCounters::missing_stage_denied();
        let failure_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_diagnostic_bundle_error_v1",
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("error_kind"),
            error_kind.as_str(),
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("message"),
            message,
        )
        .field_evidence_identity(
            crate::evidence_identity::WorthQueryEvidenceTag::new("counters"),
            &counters.evidence_identity(),
        )
        .field_value_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("evidence"),
            evidence_parts.iter().map(String::as_str),
        )
        .seal()?;
        Ok(Self {
            error_kind,
            message,
            failure_identity,
            counters,
        })
    }

    pub fn error_kind(&self) -> &QuerySubscriptionDiagnosticBundleErrorKind {
        &self.error_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.failure_identity
    }

    pub fn counters(&self) -> &QuerySubscriptionDiagnosticCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionDiagnosticFailure {
    stage: QuerySubscriptionDiagnosticStage,
    outcome: QuerySubscriptionDiagnosticOutcome,
    reason: String,
    source_identity: WorthQueryEvidenceIdentity,
    counter_identity: WorthQueryEvidenceIdentity,
    failure_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionDiagnosticFailure {
    fn new(
        stage: QuerySubscriptionDiagnosticStage,
        reason: impl Into<String>,
        source_identity: WorthQueryEvidenceIdentity,
        counter_identity: WorthQueryEvidenceIdentity,
        failure_kind: &str,
        diagnostic_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let failure_identity = diagnostic_failure_identity(failure_kind, diagnostic_identity)?;
        Ok(Self {
            stage,
            outcome: QuerySubscriptionDiagnosticOutcome::Denied,
            reason: reason.into(),
            source_identity,
            counter_identity,
            failure_identity,
        })
    }

    pub fn from_family_selection_error(
        error: &QuerySubscriptionFamilySelectionError,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            *error.diagnostic().stage(),
            error.message(),
            error.diagnostic().source_identity().clone(),
            error.counters().evidence_identity(),
            error.failure_class().as_str(),
            error.diagnostic().evidence_identity(),
        )?)
    }

    pub fn from_declaration_denial(
        error: &QuerySubscriptionDeclarationDenial,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            *error.diagnostic().stage(),
            error.message(),
            error.diagnostic().source_identity().clone(),
            error.counters().evidence_identity(),
            error.denial_kind().as_str(),
            error.diagnostic().evidence_identity(),
        )?)
    }

    pub fn from_bridge_lowering_error(
        error: &QuerySubscriptionBridgeLoweringError,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            *error.diagnostic().stage(),
            error.message(),
            error.diagnostic().source_identity().clone(),
            error.counters().evidence_identity(),
            error.denial_kind().as_str(),
            error.diagnostic().evidence_identity(),
        )?)
    }

    pub fn from_admission_error(
        error: &QuerySubscriptionAdmissionError,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            *error.pipeline_diagnostic().stage(),
            error.message(),
            error.pipeline_diagnostic().source_identity().clone(),
            error.counters().evidence_identity(),
            error.denial_kind().as_str(),
            error.pipeline_diagnostic().evidence_identity(),
        )?)
    }

    pub fn from_support_report_error(
        error: &QuerySubscriptionSupportReportError,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            QuerySubscriptionDiagnosticStage::SupportReporting,
            error.message(),
            error.failure_identity().clone(),
            error.failure_identity().clone(),
            error.denial_kind().as_str(),
            error.failure_identity(),
        )?)
    }

    pub fn from_lifecycle_certification_error(
        error: &SubscriptionLifecycleCertificationError,
    ) -> Result<Self, IdentityDerivationDenial> {
        let failure_identity = error.failure_identity().clone();
        Ok(Self::new(
            QuerySubscriptionDiagnosticStage::Certification,
            error.message(),
            failure_identity.clone(),
            failure_identity.clone(),
            "lifecycle_certification",
            &failure_identity,
        )?)
    }

    pub fn stage(&self) -> &QuerySubscriptionDiagnosticStage {
        &self.stage
    }

    pub fn outcome(&self) -> &QuerySubscriptionDiagnosticOutcome {
        &self.outcome
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.source_identity
    }

    pub fn counter_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.counter_identity
    }

    pub fn failure_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.failure_identity
    }
}
