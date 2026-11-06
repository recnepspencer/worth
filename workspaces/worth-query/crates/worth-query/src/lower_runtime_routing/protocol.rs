use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeSeamKey,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeSubjectIdentity {
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeSubjectIdentity {
    pub(crate) fn compose(
        subject_family: impl AsRef<str>,
    ) -> WorthQueryLowerRuntimeSubjectIdentityEncoder {
        WorthQueryLowerRuntimeSubjectIdentityEncoder {
            encoder: WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeCapabilitySubject,
            )
            .field_shape(WorthQueryEvidenceTag::new("subject_family"), subject_family),
        }
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }

    pub fn as_str(&self) -> &str {
        let composed = &self.evidence_identity;
        composed.reporting_projection()
    }
}

impl AsRef<str> for WorthQueryLowerRuntimeSubjectIdentity {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[must_use]
pub struct WorthQueryLowerRuntimeSubjectIdentityEncoder {
    encoder: crate::evidence_identity::WorthQueryEvidenceIdentityEncoder,
}

impl WorthQueryLowerRuntimeSubjectIdentityEncoder {
    pub(crate) fn field_value(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: impl AsRef<str>,
    ) -> Self {
        self.encoder = self.encoder.field_value(tag, value);
        self
    }

    pub(crate) fn field_evidence_identity(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: &WorthQueryEvidenceIdentity,
    ) -> Self {
        self.encoder = self.encoder.field_evidence_identity(tag, value);
        self
    }

    pub(crate) fn field_shape(
        mut self,
        tag: WorthQueryEvidenceTag,
        value: impl AsRef<str>,
    ) -> Self {
        self.encoder = self.encoder.field_shape(tag, value);
        self
    }

    pub(crate) fn field_usize(mut self, tag: WorthQueryEvidenceTag, value: usize) -> Self {
        self.encoder = self.encoder.field_usize(tag, value);
        self
    }

    pub(crate) fn seal(
        self,
    ) -> Result<WorthQueryLowerRuntimeSubjectIdentity, IdentityDerivationDenial> {
        Ok(WorthQueryLowerRuntimeSubjectIdentity {
            evidence_identity: self.encoder.seal()?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCapabilityRequest {
    seam_key: WorthQueryLowerRuntimeSeamKey,
    route_kind: WorthQueryLowerRuntimeRouteKind,
    authority_owner: WorthQueryLowerRuntimeAuthorityOwner,
    capability_label: String,
    subject_identity: WorthQueryLowerRuntimeSubjectIdentity,
    request_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeCapabilityRequest {
    pub(crate) fn new(
        seam_key: WorthQueryLowerRuntimeSeamKey,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        authority_owner: WorthQueryLowerRuntimeAuthorityOwner,
        capability_label: impl Into<String>,
        subject_identity: WorthQueryLowerRuntimeSubjectIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let capability_label = capability_label.into();
        let request_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeCapabilityRequest)
                .field_shape(WorthQueryEvidenceTag::new("seam"), seam_key.as_str())
                .field_shape(
                    WorthQueryEvidenceTag::new("route_kind"),
                    route_kind.as_str(),
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("owner"),
                    authority_owner.as_str(),
                )
                .field_shape(WorthQueryEvidenceTag::new("capability"), &capability_label)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("subject"),
                    subject_identity.evidence_identity(),
                )
                .seal()?;
        Ok(Self {
            seam_key,
            route_kind,
            authority_owner,
            capability_label,
            subject_identity,
            request_identity,
        })
    }

    pub fn seam_key(&self) -> WorthQueryLowerRuntimeSeamKey {
        self.seam_key
    }

    pub fn route_kind(&self) -> WorthQueryLowerRuntimeRouteKind {
        self.route_kind
    }

    pub fn authority_owner(&self) -> WorthQueryLowerRuntimeAuthorityOwner {
        self.authority_owner
    }

    pub fn capability_label(&self) -> &str {
        &self.capability_label
    }

    pub fn subject_digest(&self) -> &str {
        self.subject_identity.as_str()
    }

    pub fn subject_identity(&self) -> &WorthQueryLowerRuntimeSubjectIdentity {
        &self.subject_identity
    }

    pub fn request_digest(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn drift_from_contract(
        &self,
        seam_key: WorthQueryLowerRuntimeSeamKey,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        authority_owner: WorthQueryLowerRuntimeAuthorityOwner,
        capability_label: &str,
        subject_identity: &WorthQueryLowerRuntimeSubjectIdentity,
    ) -> Option<String> {
        if self.seam_key != seam_key {
            return Some("lower-runtime capability request seam key drifted".to_string());
        }
        if self.route_kind != route_kind {
            return Some("lower-runtime capability request route kind drifted".to_string());
        }
        if self.authority_owner != authority_owner {
            return Some("lower-runtime capability request authority owner drifted".to_string());
        }
        if self.capability_label != capability_label {
            return Some("lower-runtime capability request capability label drifted".to_string());
        }
        if &self.subject_identity != subject_identity {
            return Some("lower-runtime capability request subject identity drifted".to_string());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_digest_binds_seam_route_owner_and_subject() -> Result<(), IdentityDerivationDenial> {
        let request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            "write-authority",
            WorthQueryLowerRuntimeSubjectIdentity::compose("test-subject")
                .field_value(WorthQueryEvidenceTag::new("test_subject"), "subject-1")
                .seal()?,
        )?;

        assert_eq!(
            request.seam_key(),
            WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution
        );
        assert_eq!(
            request.route_kind(),
            WorthQueryLowerRuntimeRouteKind::RoutePlanning
        );
        assert_eq!(
            request.authority_owner(),
            WorthQueryLowerRuntimeAuthorityOwner::Query
        );
        assert_eq!(request.capability_label(), "write-authority");
        assert_eq!(
            request.subject_identity().evidence_identity().scope(),
            WorthQueryEvidenceScope::LowerRuntimeCapabilitySubject
        );
        assert!(!request.request_digest().is_empty());

        Ok(())
    }
}
