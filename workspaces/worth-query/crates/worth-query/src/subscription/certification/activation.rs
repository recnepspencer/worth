use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::activation::SubscriptionActivationInput;
use super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::evidence_identities::{
    certification_activation_bundle_identity, typed_identity_drift,
};
use super::super::scale::QuerySubscriptionScaleSlopeReport;
use super::super::validation_evidence::validation_role_evidence_identity;
use super::identity::subscription_certification_failure_identity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionCertificationDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    ActivationAdmissionMismatch,
    ScaleSlopeDrift,
    ScaleSlopeSourceMismatch,
}

impl QuerySubscriptionCertificationDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::ActivationAdmissionMismatch => "activation_admission_mismatch",
            Self::ScaleSlopeDrift => "scale_slope_drift",
            Self::ScaleSlopeSourceMismatch => "scale_slope_source_mismatch",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionCertificationError {
    denial_kind: QuerySubscriptionCertificationDenialKind,
    message: &'static str,
    failure_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl QuerySubscriptionCertificationError {
    pub(in crate::subscription) fn new(
        denial_kind: QuerySubscriptionCertificationDenialKind,
        message: &'static str,
        evidence: &[WorthQueryEvidenceIdentity],
    ) -> Self {
        Self {
            denial_kind,
            message,
            failure_identity: (|| {
                Ok(subscription_certification_failure_identity(
                    "query_subscription_certification_error_v1",
                    denial_kind.as_str(),
                    message,
                    evidence,
                )?)
            })(),
        }
    }

    pub fn denial_kind(&self) -> &QuerySubscriptionCertificationDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.failure_identity.as_ref().map_err(|cause| *cause)?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionCertificationBundle {
    pub(in crate::subscription) certification_bundle_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) admission_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) activation_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) query_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) bridge_declaration_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) basis_binding_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) signal_strategy_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) diagnostics_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) support_profile_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) admission_counter_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) activation_counter_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) scale_slope_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) scale_activation_identity: WorthQueryEvidenceIdentity,
    pub(in crate::subscription) scale_admission_identity: WorthQueryEvidenceIdentity,
}

pub fn certify_query_subscription_activation(
    admission: QuerySubscriptionAdmissionArtifact,
    activation: SubscriptionActivationInput,
    scale_report: QuerySubscriptionScaleSlopeReport,
) -> Result<QuerySubscriptionCertificationBundle, QuerySubscriptionCertificationError> {
    if typed_identity_drift(
        activation.admission_identity(),
        admission.evidence_identity(),
    ) || typed_identity_drift(
        activation.query_declaration_identity(),
        admission.query_declaration_identity(),
    ) || typed_identity_drift(
        activation.bridge_declaration_identity(),
        admission.bridge_declaration_identity(),
    ) || typed_identity_drift(
        activation.basis_binding_identity(),
        admission.basis_binding_identity(),
    ) || typed_identity_drift(
        activation.signal_strategy_identity(),
        admission.signal_strategy_identity(),
    ) {
        return Err(QuerySubscriptionCertificationError::new(
            QuerySubscriptionCertificationDenialKind::ActivationAdmissionMismatch,
            "subscription activation input does not match the admitted subscription artifact",
            &[
                validation_role_evidence_identity("admission", admission.evidence_identity())?,
                validation_role_evidence_identity(
                    "activation_admission",
                    activation.admission_identity(),
                )?,
                validation_role_evidence_identity(
                    "admission_query",
                    admission.query_declaration_identity(),
                )?,
                validation_role_evidence_identity(
                    "activation_query",
                    activation.query_declaration_identity(),
                )?,
            ],
        ));
    }

    if typed_identity_drift(
        scale_report.activation_identity(),
        activation.evidence_identity(),
    ) || typed_identity_drift(
        scale_report.admission_identity(),
        activation.admission_identity(),
    ) {
        return Err(QuerySubscriptionCertificationError::new(
            QuerySubscriptionCertificationDenialKind::ScaleSlopeSourceMismatch,
            "subscription scale slope report does not certify this activation source",
            &[
                validation_role_evidence_identity("activation", activation.evidence_identity())?,
                validation_role_evidence_identity(
                    "scale_activation",
                    scale_report.activation_identity(),
                )?,
                validation_role_evidence_identity(
                    "activation_admission",
                    activation.admission_identity(),
                )?,
                validation_role_evidence_identity(
                    "scale_admission",
                    scale_report.admission_identity(),
                )?,
            ],
        ));
    }

    let admission_counter_identity = admission.counters().evidence_identity();
    let activation_counter_identity = activation.counters().evidence_identity();
    let diagnostics_identity = admission.diagnostics().diagnostics_identity().clone();
    let support_profile_identity = admission.support_profile().profile_identity().clone();
    let scale_slope_identity = scale_report.evidence_identity_ref().clone();
    let scale_activation_identity = scale_report.activation_identity().clone();
    let scale_admission_identity = scale_report.admission_identity().clone();
    let certification_bundle_identity = certification_activation_bundle_identity(
        admission.evidence_identity(),
        activation.evidence_identity(),
        admission.query_declaration_identity(),
        admission.bridge_declaration_identity(),
        admission.basis_binding_identity(),
        admission.signal_strategy_identity(),
        admission.diagnostics().diagnostics_identity(),
        admission.support_profile().profile_identity(),
        &admission.counters().evidence_identity(),
        &activation.counters().evidence_identity(),
        scale_report.evidence_identity_ref(),
    );
    Ok(QuerySubscriptionCertificationBundle {
        certification_bundle_identity,
        admission_identity: admission.evidence_identity().clone(),
        activation_identity: activation.evidence_identity().clone(),
        query_declaration_identity: admission.query_declaration_identity().clone(),
        bridge_declaration_identity: admission.bridge_declaration_identity().clone(),
        basis_binding_identity: admission.basis_binding_identity().clone(),
        signal_strategy_identity: admission.signal_strategy_identity().clone(),
        diagnostics_identity,
        support_profile_identity,
        admission_counter_identity,
        activation_counter_identity,
        scale_slope_identity,
        scale_activation_identity,
        scale_admission_identity,
    })
}

impl From<IdentityDerivationDenial> for QuerySubscriptionCertificationError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            denial_kind: QuerySubscriptionCertificationDenialKind::EvidenceIdentityDerivationDenied(
                cause,
            ),
            message: "subscription activation certification identity refused",
            failure_identity: Err(cause),
        }
    }
}
