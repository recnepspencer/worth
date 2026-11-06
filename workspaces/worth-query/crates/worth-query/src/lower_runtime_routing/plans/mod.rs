use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::WorthQueryLowerRuntimeCapabilityEligibility;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeRouteSubjectIdentity {
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeRouteSubjectIdentity {
    pub(crate) fn from_evidence_identity(
        route_family: impl AsRef<str>,
        subject_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            evidence_identity: WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeRouteSubject,
            )
            .field_shape(WorthQueryEvidenceTag::new("route_family"), route_family)
            .field_evidence_identity(WorthQueryEvidenceTag::new("subject"), subject_identity)
            .seal()?,
        })
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }

    pub fn as_str(&self) -> &str {
        let composed = &self.evidence_identity;
        composed.reporting_projection()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeRoutePlan {
    eligibility: WorthQueryLowerRuntimeCapabilityEligibility,
    route_subject: WorthQueryLowerRuntimeRouteSubjectIdentity,
    route_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeRoutePlan {
    pub(crate) fn new(
        eligibility: WorthQueryLowerRuntimeCapabilityEligibility,
        route_subject: WorthQueryLowerRuntimeRouteSubjectIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let route_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeRoutePlan)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("eligibility"),
                    eligibility.eligibility_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("route_subject"),
                    route_subject.evidence_identity(),
                )
                .seal()?;
        Ok(Self {
            eligibility,
            route_subject,
            route_identity,
        })
    }

    pub fn eligibility(&self) -> &WorthQueryLowerRuntimeCapabilityEligibility {
        &self.eligibility
    }

    pub fn route_subject(&self) -> &WorthQueryLowerRuntimeRouteSubjectIdentity {
        &self.route_subject
    }

    pub fn route_digest(&self) -> &str {
        self.route_identity.as_str()
    }

    pub fn route_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.route_identity
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower_runtime_routing::{
        WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeCapabilityRequest,
        WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeSeamKey,
        WorthQueryLowerRuntimeSubjectIdentity,
    };

    #[test]
    fn route_plan_digest_reuses_eligibility_digest() -> Result<(), IdentityDerivationDenial> {
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
        let plan = WorthQueryLowerRuntimeRoutePlan::new(
            eligibility,
            WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
                "test-route",
                &detail_identity,
            )?,
        )?;

        assert_eq!(
            plan.route_subject().evidence_identity().scope(),
            WorthQueryEvidenceScope::LowerRuntimeRouteSubject
        );
        assert!(!plan.route_digest().is_empty());

        Ok(())
    }
}
