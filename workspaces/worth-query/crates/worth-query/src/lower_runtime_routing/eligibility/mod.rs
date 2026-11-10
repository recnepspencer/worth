use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::WorthQueryLowerRuntimeCapabilityRequest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeCapabilityEligibility {
    request: WorthQueryLowerRuntimeCapabilityRequest,
    posture: WorthQueryLowerRuntimeCapabilityPosture,
    posture_detail_identity: WorthQueryEvidenceIdentity,
    eligibility_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeCapabilityEligibility {
    pub(crate) fn admitted_with_evidence_identity(
        request: WorthQueryLowerRuntimeCapabilityRequest,
        posture_detail_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new_with_evidence_identity(
            request,
            WorthQueryLowerRuntimeCapabilityPosture::Admitted,
            posture_detail_identity,
        )
    }

    pub(crate) fn new_with_evidence_identity(
        request: WorthQueryLowerRuntimeCapabilityRequest,
        posture: WorthQueryLowerRuntimeCapabilityPosture,
        posture_detail_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let eligibility_identity = Self::digest_builder(&request, posture)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("detail"),
                posture_detail_identity,
            )
            .seal()?;
        Ok(Self {
            request,
            posture,
            posture_detail_identity: posture_detail_identity.clone(),
            eligibility_identity,
        })
    }

    fn digest_builder(
        request: &WorthQueryLowerRuntimeCapabilityRequest,
        posture: WorthQueryLowerRuntimeCapabilityPosture,
    ) -> crate::evidence_identity::WorthQueryEvidenceIdentityEncoder {
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeCapabilityEligibility)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("request"),
                request.request_identity(),
            )
            .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
    }

    pub fn request(&self) -> &WorthQueryLowerRuntimeCapabilityRequest {
        &self.request
    }

    pub fn posture(&self) -> WorthQueryLowerRuntimeCapabilityPosture {
        self.posture
    }

    pub fn posture_detail_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.posture_detail_identity
    }

    pub fn eligibility_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.eligibility_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeCapabilityPosture {
    Admitted,
    Deferred,
    Unsupported,
    Forbidden,
}

impl WorthQueryLowerRuntimeCapabilityPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Deferred => "deferred",
            Self::Unsupported => "unsupported",
            Self::Forbidden => "forbidden",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower_runtime_routing::{
        WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeRouteKind,
        WorthQueryLowerRuntimeSeamKey, WorthQueryLowerRuntimeSubjectIdentity,
    };

    #[test]
    fn eligibility_digest_binds_request_posture_and_detail() -> Result<(), IdentityDerivationDenial>
    {
        let request = WorthQueryLowerRuntimeCapabilityRequest::new(
            WorthQueryLowerRuntimeSeamKey::WriteAuthorityBackendExecution,
            WorthQueryLowerRuntimeRouteKind::RoutePlanning,
            WorthQueryLowerRuntimeAuthorityOwner::Query,
            "write-authority",
            WorthQueryLowerRuntimeSubjectIdentity::compose("test-subject")
                .field_value(WorthQueryEvidenceTag::new("test_subject"), "subject-1")
                .seal()?,
        )?;

        let detail_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
        )
        .field_value(WorthQueryEvidenceTag::new("test_detail"), "detail-1")
        .seal()?;
        let eligibility =
            WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
                request,
                &detail_identity,
            )?;

        assert_eq!(eligibility.posture().as_str(), "admitted");
        let eligibility_identity = eligibility.eligibility_identity();
        assert!(!eligibility_identity.reporting_projection().is_empty());

        Ok(())
    }
}
