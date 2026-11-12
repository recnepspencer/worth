use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity_authority::{QueryProjectionIdentity, QuerySubscriptionIdentityKind};

use super::super::super::evidence_projection::subscription_evidence_projection;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionSupportReportDenialKind {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    DeclarationSourceMismatch,
    FamilySourceMismatch,
    AdmissionSourceMismatch,
    AdmissionEvidenceRequired,
}

impl QuerySubscriptionSupportReportDenialKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::DeclarationSourceMismatch => "declaration_source_mismatch",
            Self::FamilySourceMismatch => "family_source_mismatch",
            Self::AdmissionSourceMismatch => "admission_source_mismatch",
            Self::AdmissionEvidenceRequired => "admission_evidence_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportReportError {
    denial_kind: QuerySubscriptionSupportReportDenialKind,
    message: &'static str,
    failure_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl QuerySubscriptionSupportReportError {
    pub(super) fn new(
        denial_kind: QuerySubscriptionSupportReportDenialKind,
        message: &'static str,
        evidence_parts: &[String],
    ) -> Self {
        let failure_identity = WorthQueryEvidenceIdentity::compose(
            crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_support_report_error_v1",
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("denial_kind"),
            denial_kind.as_str(),
        )
        .field_shape(
            crate::evidence_identity::WorthQueryEvidenceTag::new("message"),
            message,
        )
        .field_value_sequence(
            crate::evidence_identity::WorthQueryEvidenceTag::new("evidence"),
            evidence_parts.iter().map(String::as_str),
        )
        .seal();
        Self {
            denial_kind,
            message,
            failure_identity,
        }
    }

    pub fn denial_kind(&self) -> &QuerySubscriptionSupportReportDenialKind {
        &self.denial_kind
    }

    pub fn message(&self) -> &str {
        self.message
    }

    pub fn failure_projection(
        &self,
    ) -> Result<
        QueryProjectionIdentity<String, QuerySubscriptionIdentityKind>,
        IdentityDerivationDenial,
    > {
        Ok(subscription_evidence_projection(
            self.failure_identity.as_ref().map_err(|cause| *cause)?,
        ))
    }

    pub fn failure_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.failure_identity.as_ref().map_err(|cause| *cause)
    }
}

impl From<IdentityDerivationDenial> for QuerySubscriptionSupportReportError {
    fn from(cause: IdentityDerivationDenial) -> Self {
        Self {
            denial_kind: QuerySubscriptionSupportReportDenialKind::EvidenceIdentityDerivationDenied(
                cause,
            ),
            message: "subscription support report identity refused",
            failure_identity: Err(cause),
        }
    }
}
