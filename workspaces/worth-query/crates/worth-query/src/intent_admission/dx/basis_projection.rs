use crate::basis_lifecycle::{BasisLifecycleIntentDraft, ScopedObservationBasis};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    WorthQueryBasisObservationPlan, WorthQueryProjectionConsumptionPlan,
};
use crate::projection_consumption::{
    MaterializedProjectionContract, ProjectionConsumptionDeclaration,
};
use crate::runtime::WorthQueryIntentConsumerInspection;

use super::{
    WorthQueryAdmittedIntentPlan, WorthQueryIntentAdmissionDecision,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentDecisionTraceEnvelope,
    WorthQueryIntentNonAdmittedStop, WorthQueryRawIntentAdmissionRequest,
    WorthQueryRuntimeIntentAdmissionReviewData,
};

pub fn worth_query_basis_observation_intent(
    declaration: BasisLifecycleIntentDraft,
) -> Result<WorthQueryBasisObservationIntentAuthoring, super::WorthQueryIntentViolationDecision> {
    Ok(WorthQueryBasisObservationIntentAuthoring {
        request: WorthQueryRawIntentAdmissionRequest::basis_observation_lane(
            declaration.into_raw(),
        )?,
    })
}

pub fn worth_query_projection_consumption_intent(
    declaration: ProjectionConsumptionDeclaration,
) -> Result<WorthQueryProjectionConsumptionIntentAuthoring, super::WorthQueryIntentViolationDecision>
{
    Ok(WorthQueryProjectionConsumptionIntentAuthoring {
        request: WorthQueryRawIntentAdmissionRequest::projection_consumption(declaration)?,
    })
}

pub struct WorthQueryBasisObservationIntentAuthoring {
    request: WorthQueryRawIntentAdmissionRequest,
}

impl WorthQueryBasisObservationIntentAuthoring {
    pub fn review(
        self,
    ) -> Result<WorthQueryBasisObservationIntentReview, IdentityDerivationDenial> {
        Ok(WorthQueryBasisObservationIntentReview {
            review: WorthQueryRuntimeIntentAdmissionReviewData::from_request(self.request)?,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryBasisObservationAdmittedIntent, WorthQueryIntentNonAdmittedStop> {
        let family = self.request.family();
        let entrypoint = self.request.entrypoint();
        self.review()
            .map_err(|cause| {
                WorthQueryIntentNonAdmittedStop::Violation(
                    super::WorthQueryIntentViolationDecision::identity_refused(
                        family, entrypoint, cause,
                    )
                    .into_stop(),
                )
            })?
            .admit()
    }
}

pub struct WorthQueryBasisObservationIntentReview {
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl WorthQueryBasisObservationIntentReview {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }
    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }
    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }
    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.review.decision_trace_envelope()
    }
    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_review(
            self.review.request().intent_name(),
            self.review.decision(),
            self.review.request().family(),
            self.review.request().entrypoint(),
            self.review.decision_trace_envelope(),
        )
    }
    pub fn admit(
        self,
    ) -> Result<WorthQueryBasisObservationAdmittedIntent, WorthQueryIntentNonAdmittedStop> {
        match self.review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                WorthQueryAdmittedIntentPlan::BasisObservation(plan),
            ) => Ok(WorthQueryBasisObservationAdmittedIntent { plan }),
            _ => Err(self
                .review
                .non_admitted_stop()
                .expect("non-admitted basis observation review should stop")),
        }
    }
}

pub struct WorthQueryBasisObservationAdmittedIntent {
    plan: WorthQueryBasisObservationPlan,
}

impl WorthQueryBasisObservationAdmittedIntent {
    pub fn plan(&self) -> &WorthQueryBasisObservationPlan {
        &self.plan
    }

    pub fn scope(self) -> ScopedObservationBasis {
        self.plan.scope()
    }
}

pub struct WorthQueryProjectionConsumptionIntentAuthoring {
    request: WorthQueryRawIntentAdmissionRequest,
}

impl WorthQueryProjectionConsumptionIntentAuthoring {
    pub fn review(
        self,
    ) -> Result<WorthQueryProjectionConsumptionIntentReview, IdentityDerivationDenial> {
        Ok(WorthQueryProjectionConsumptionIntentReview {
            review: WorthQueryRuntimeIntentAdmissionReviewData::from_request(self.request)?,
        })
    }

    pub fn admit(
        self,
    ) -> Result<WorthQueryProjectionConsumptionAdmittedIntent, WorthQueryIntentNonAdmittedStop>
    {
        let family = self.request.family();
        let entrypoint = self.request.entrypoint();
        self.review()
            .map_err(|cause| {
                WorthQueryIntentNonAdmittedStop::Violation(
                    super::WorthQueryIntentViolationDecision::identity_refused(
                        family, entrypoint, cause,
                    )
                    .into_stop(),
                )
            })?
            .admit()
    }
}

pub struct WorthQueryProjectionConsumptionIntentReview {
    review: WorthQueryRuntimeIntentAdmissionReviewData,
}

impl WorthQueryProjectionConsumptionIntentReview {
    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        self.review.request()
    }
    pub fn eligibility(&self) -> &WorthQueryIntentAdmissionEligibility {
        self.review.eligibility()
    }
    pub fn decision(&self) -> &WorthQueryIntentAdmissionDecision {
        self.review.decision()
    }
    pub fn decision_trace_envelope(&self) -> Option<&WorthQueryIntentDecisionTraceEnvelope> {
        self.review.decision_trace_envelope()
    }
    pub fn consumer_inspection(&self) -> WorthQueryIntentConsumerInspection<'_> {
        WorthQueryIntentConsumerInspection::from_review(
            self.review.request().intent_name(),
            self.review.decision(),
            self.review.request().family(),
            self.review.request().entrypoint(),
            self.review.decision_trace_envelope(),
        )
    }
    pub fn admit(
        self,
    ) -> Result<WorthQueryProjectionConsumptionAdmittedIntent, WorthQueryIntentNonAdmittedStop>
    {
        match self.review.decision().clone() {
            WorthQueryIntentAdmissionDecision::Admitted(
                WorthQueryAdmittedIntentPlan::ProjectionConsumption(plan),
            ) => Ok(WorthQueryProjectionConsumptionAdmittedIntent { plan }),
            _ => Err(self
                .review
                .non_admitted_stop()
                .expect("non-admitted projection review should stop")),
        }
    }
}

pub struct WorthQueryProjectionConsumptionAdmittedIntent {
    plan: WorthQueryProjectionConsumptionPlan,
}

impl WorthQueryProjectionConsumptionAdmittedIntent {
    pub fn plan(&self) -> &WorthQueryProjectionConsumptionPlan {
        &self.plan
    }

    pub fn bind_contract(
        &self,
    ) -> Result<MaterializedProjectionContract, IdentityDerivationDenial> {
        self.plan.bind_contract()
    }
}
