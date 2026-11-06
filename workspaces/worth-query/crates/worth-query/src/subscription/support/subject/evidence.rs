use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::super::super::admission::QuerySubscriptionAdmissionArtifact;
use super::super::super::declaration::QuerySubscriptionDeclarationArtifact;
use super::super::super::family::QuerySubscriptionFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportEvidence {
    kind: QuerySubscriptionSupportEvidenceKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionSupportEvidenceError {
    message: &'static str,
    pub(in crate::subscription::support) failure_identity:
        Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl QuerySubscriptionSupportEvidenceError {
    fn new(
        message: &'static str,
        declaration_identity: &WorthQueryEvidenceIdentity,
        admission_declaration_identity: &WorthQueryEvidenceIdentity,
    ) -> Self {
        let failure_identity = (|| {
            Ok(WorthQueryEvidenceIdentity::compose(
                crate::evidence_identity::WorthQueryEvidenceScope::SubscriptionActivationReceipt,
            )
            .field_shape(
                crate::evidence_identity::WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_support_evidence_error_v1",
            )
            .field_shape(
                crate::evidence_identity::WorthQueryEvidenceTag::new("message"),
                message,
            )
            .field_evidence_identity(
                crate::evidence_identity::WorthQueryEvidenceTag::new("declaration"),
                declaration_identity,
            )
            .field_evidence_identity(
                crate::evidence_identity::WorthQueryEvidenceTag::new("admission_declaration"),
                admission_declaration_identity,
            )
            .seal()?)
        })();
        Self {
            message,
            failure_identity,
        }
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
enum QuerySubscriptionSupportEvidenceKind {
    Declaration {
        declaration_identity: WorthQueryEvidenceIdentity,
        family: QuerySubscriptionFamily,
        source_identity: WorthQueryEvidenceIdentity,
    },
    Admission {
        declaration_identity: WorthQueryEvidenceIdentity,
        family: QuerySubscriptionFamily,
        admission_identity: WorthQueryEvidenceIdentity,
        support_profile: super::super::profile::QuerySubscriptionSupportProfile,
        source_identity: WorthQueryEvidenceIdentity,
    },
}

impl QuerySubscriptionSupportEvidence {
    pub fn declaration(declaration: &QuerySubscriptionDeclarationArtifact) -> Self {
        Self {
            kind: QuerySubscriptionSupportEvidenceKind::Declaration {
                declaration_identity: declaration.declaration_identity().clone(),
                family: declaration.family().clone(),
                source_identity: declaration.declaration_identity().clone(),
            },
        }
    }

    pub fn admission(
        declaration: &QuerySubscriptionDeclarationArtifact,
        admission: &QuerySubscriptionAdmissionArtifact,
    ) -> Result<Self, QuerySubscriptionSupportEvidenceError> {
        if crate::subscription::evidence_identities::typed_identity_drift(
            declaration.declaration_identity(),
            admission.query_declaration_identity(),
        ) {
            return Err(QuerySubscriptionSupportEvidenceError::new(
                "subscription support evidence requires declaration and admission artifacts from the same canonical query subscription family",
                declaration.declaration_identity(),
                admission.query_declaration_identity(),
            ));
        }

        Ok(Self {
            kind: QuerySubscriptionSupportEvidenceKind::Admission {
                declaration_identity: declaration.declaration_identity().clone(),
                family: declaration.family().clone(),
                admission_identity: admission.evidence_identity().clone(),
                support_profile: admission.support_profile().clone(),
                source_identity: admission.evidence_identity().clone(),
            },
        })
    }

    pub(crate) fn declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        match &self.kind {
            QuerySubscriptionSupportEvidenceKind::Declaration {
                declaration_identity,
                ..
            }
            | QuerySubscriptionSupportEvidenceKind::Admission {
                declaration_identity,
                ..
            } => declaration_identity,
        }
    }

    pub(crate) fn family(&self) -> &QuerySubscriptionFamily {
        match &self.kind {
            QuerySubscriptionSupportEvidenceKind::Declaration { family, .. }
            | QuerySubscriptionSupportEvidenceKind::Admission { family, .. } => family,
        }
    }

    pub(crate) fn admission_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        match &self.kind {
            QuerySubscriptionSupportEvidenceKind::Declaration { .. } => None,
            QuerySubscriptionSupportEvidenceKind::Admission {
                admission_identity, ..
            } => Some(admission_identity),
        }
    }

    pub(crate) fn support_profile(
        &self,
    ) -> Option<&super::super::profile::QuerySubscriptionSupportProfile> {
        match &self.kind {
            QuerySubscriptionSupportEvidenceKind::Declaration { .. } => None,
            QuerySubscriptionSupportEvidenceKind::Admission {
                support_profile, ..
            } => Some(support_profile),
        }
    }

    pub(crate) fn source_identity(&self) -> &WorthQueryEvidenceIdentity {
        match &self.kind {
            QuerySubscriptionSupportEvidenceKind::Declaration {
                source_identity, ..
            }
            | QuerySubscriptionSupportEvidenceKind::Admission {
                source_identity, ..
            } => source_identity,
        }
    }
}
