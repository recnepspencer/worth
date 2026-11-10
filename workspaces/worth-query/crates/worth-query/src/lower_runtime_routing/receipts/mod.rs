use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{WorthQueryLowerRuntimeCapabilityEligibility, WorthQueryLowerRuntimeRoutePlan};

pub(crate) fn worth_query_lower_runtime_retained_evidence_identity(
    retained_family: impl AsRef<str>,
    retained_evidence: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryLowerRuntimeRetainedEvidenceIdentity, IdentityDerivationDenial> {
    Ok(WorthQueryLowerRuntimeRetainedEvidenceIdentity::new(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("retained_family"),
                retained_family,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("retained"), retained_evidence)
            .seal()?,
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeRetainedEvidenceIdentity {
    evidence_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeRetainedEvidenceIdentity {
    pub(crate) fn new(evidence_identity: WorthQueryEvidenceIdentity) -> Self {
        Self { evidence_identity }
    }

    pub(crate) fn from_evidence_identity(
        retained_family: impl AsRef<str>,
        evidence_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        worth_query_lower_runtime_retained_evidence_identity(retained_family, evidence_identity)
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.evidence_identity
    }

    pub fn as_str(&self) -> &str {
        let composed = &self.evidence_identity;
        composed.reporting_projection()
    }
}

impl AsRef<str> for WorthQueryLowerRuntimeRetainedEvidenceIdentity {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeReadmissionReceipt {
    eligibility: WorthQueryLowerRuntimeCapabilityEligibility,
    retained_evidence_identity: WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    handoff_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeReadmissionReceipt {
    pub(crate) fn new(
        eligibility: WorthQueryLowerRuntimeCapabilityEligibility,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Self {
        let handoff_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeReadmissionReceipt)
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("eligibility"),
                    eligibility.eligibility_identity(),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("retained_evidence"),
                    retained_evidence_identity.evidence_identity(),
                )
                .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens");
        Self {
            eligibility,
            retained_evidence_identity: retained_evidence_identity.clone(),
            handoff_identity,
        }
    }

    pub fn eligibility(&self) -> &WorthQueryLowerRuntimeCapabilityEligibility {
        &self.eligibility
    }

    pub fn retained_evidence_identity(&self) -> &WorthQueryLowerRuntimeRetainedEvidenceIdentity {
        &self.retained_evidence_identity
    }

    pub fn handoff_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.handoff_identity
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryLowerRuntimeBoundaryExecutionKind {
    RoutePlan,
    ReadmissionHandoff,
}

impl WorthQueryLowerRuntimeBoundaryExecutionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RoutePlan => "route-plan",
            Self::ReadmissionHandoff => "readmission-handoff",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryLowerRuntimeBoundaryExecutionReceipt {
    kind: WorthQueryLowerRuntimeBoundaryExecutionKind,
    request_identity: WorthQueryEvidenceIdentity,
    eligibility_identity: WorthQueryEvidenceIdentity,
    route_or_handoff_identity: WorthQueryEvidenceIdentity,
    retained_evidence_identity: WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    boundary_execution_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryLowerRuntimeBoundaryExecutionReceipt {
    pub(crate) fn from_route_plan(
        plan: &WorthQueryLowerRuntimeRoutePlan,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Self {
        Self::new(
            WorthQueryLowerRuntimeBoundaryExecutionKind::RoutePlan,
            plan.eligibility().request().request_identity(),
            plan.eligibility().eligibility_identity(),
            plan.route_identity(),
            retained_evidence_identity,
        )
    }

    pub(crate) fn from_route_plan_with_retained_evidence_identity(
        plan: &WorthQueryLowerRuntimeRoutePlan,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Self {
        Self::from_route_plan(plan, retained_evidence_identity)
    }

    pub(crate) fn from_readmission_receipt(
        receipt: &WorthQueryLowerRuntimeReadmissionReceipt,
    ) -> Self {
        Self::new(
            WorthQueryLowerRuntimeBoundaryExecutionKind::ReadmissionHandoff,
            receipt.eligibility().request().request_identity(),
            receipt.eligibility().eligibility_identity(),
            receipt.handoff_identity(),
            receipt.retained_evidence_identity(),
        )
    }

    fn new(
        kind: WorthQueryLowerRuntimeBoundaryExecutionKind,
        request_identity: &WorthQueryEvidenceIdentity,
        eligibility_identity: &WorthQueryEvidenceIdentity,
        route_or_handoff_identity: &WorthQueryEvidenceIdentity,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Self {
        let boundary_execution_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::LowerRuntimeBoundaryExecutionReceipt,
        )
        .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
        .field_evidence_identity(WorthQueryEvidenceTag::new("request"), request_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("eligibility"),
            eligibility_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("route_or_handoff"),
            route_or_handoff_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("retained_evidence"),
            retained_evidence_identity.evidence_identity(),
        )
        .seal().expect("fixed digest grammar: literal tags, primitive values, closed enum labels, or sealed digest tokens");
        Self {
            kind,
            request_identity: request_identity.clone(),
            eligibility_identity: eligibility_identity.clone(),
            route_or_handoff_identity: route_or_handoff_identity.clone(),
            retained_evidence_identity: retained_evidence_identity.clone(),
            boundary_execution_identity,
        }
    }

    pub fn kind(&self) -> WorthQueryLowerRuntimeBoundaryExecutionKind {
        self.kind
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn eligibility_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.eligibility_identity
    }

    pub fn route_or_handoff_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.route_or_handoff_identity
    }

    pub fn retained_evidence_identity(&self) -> &WorthQueryLowerRuntimeRetainedEvidenceIdentity {
        &self.retained_evidence_identity
    }

    pub fn boundary_execution_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.boundary_execution_identity
    }

    pub fn drift_from_route_plan(
        &self,
        plan: &WorthQueryLowerRuntimeRoutePlan,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Option<String> {
        if self.kind != WorthQueryLowerRuntimeBoundaryExecutionKind::RoutePlan {
            return Some("boundary execution kind drifted from route-plan".to_string());
        }
        if self.request_identity != *plan.eligibility().request().request_identity() {
            return Some(
                "boundary execution request digest drifted from the route plan".to_string(),
            );
        }
        if self.eligibility_identity != *plan.eligibility().eligibility_identity() {
            return Some(
                "boundary execution eligibility digest drifted from the route plan".to_string(),
            );
        }
        if self.route_or_handoff_identity != *plan.route_identity() {
            return Some("boundary execution route digest drifted from the route plan".to_string());
        }
        if self.retained_evidence_identity != *retained_evidence_identity {
            return Some(
                "boundary execution retained evidence drifted from the routed evidence".to_string(),
            );
        }
        None
    }

    pub fn drift_from_route_plan_with_retained_evidence_identity(
        &self,
        plan: &WorthQueryLowerRuntimeRoutePlan,
        retained_evidence_identity: &WorthQueryLowerRuntimeRetainedEvidenceIdentity,
    ) -> Option<String> {
        self.drift_from_route_plan(plan, retained_evidence_identity)
    }

    pub fn drift_from_readmission_receipt(
        &self,
        receipt: &WorthQueryLowerRuntimeReadmissionReceipt,
    ) -> Option<String> {
        if self.kind != WorthQueryLowerRuntimeBoundaryExecutionKind::ReadmissionHandoff {
            return Some("boundary execution kind drifted from readmission-handoff".to_string());
        }
        if self.request_identity != *receipt.eligibility().request().request_identity() {
            return Some(
                "boundary execution request digest drifted from the readmission receipt"
                    .to_string(),
            );
        }
        if self.eligibility_identity != *receipt.eligibility().eligibility_identity() {
            return Some(
                "boundary execution eligibility digest drifted from the readmission receipt"
                    .to_string(),
            );
        }
        if self.route_or_handoff_identity != *receipt.handoff_identity() {
            return Some(
                "boundary execution handoff digest drifted from the readmission receipt"
                    .to_string(),
            );
        }
        if self.retained_evidence_identity != *receipt.retained_evidence_identity() {
            return Some(
                "boundary execution retained evidence drifted from the readmission receipt"
                    .to_string(),
            );
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lower_runtime_routing::{
        WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeCapabilityRequest,
        WorthQueryLowerRuntimeRouteKind, WorthQueryLowerRuntimeRouteSubjectIdentity,
        WorthQueryLowerRuntimeSeamKey, WorthQueryLowerRuntimeSubjectIdentity,
    };

    #[test]
    fn route_plan_boundary_receipt_reuses_request_and_eligibility_digests(
    ) -> Result<(), IdentityDerivationDenial> {
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
        );
        let retained_evidence = worth_query_lower_runtime_retained_evidence_identity(
            "receipt-test",
            &WorthQueryEvidenceIdentity::compose(
                WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence,
            )
            .field_value(WorthQueryEvidenceTag::new("test_retained"), "receipt-1")
            .seal()?,
        )?;
        let boundary = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
            &plan,
            &retained_evidence,
        );

        assert_eq!(
            boundary.request_identity(),
            plan.eligibility().request().request_identity()
        );
        assert_eq!(
            boundary.eligibility_identity(),
            plan.eligibility().eligibility_identity()
        );
        assert_eq!(
            boundary.kind(),
            WorthQueryLowerRuntimeBoundaryExecutionKind::RoutePlan
        );

        Ok(())
    }
}
