use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;

use super::facts::{
    WorthQueryIntentAdmissionAuthorityLaneEligibility, WorthQueryIntentAdmissionBasisEligibility,
    WorthQueryIntentAdmissionCapabilityEligibility, WorthQueryIntentAdmissionInvariantEligibility,
    WorthQueryIntentAdmissionPolicyEligibility,
    WorthQueryIntentAdmissionProjectionSourceEligibility,
    WorthQueryIntentAdmissionRoutingSupportEligibility,
    WorthQueryIntentAdmissionSourceLaneEligibility, WorthQueryIntentAdmissionSupportEligibility,
};
use super::{request::WorthQueryRawIntentAdmissionRequest, resolution::resolve_eligibility_facts};
use crate::intent_admission::WorthQueryIntentAdmissionExecutionSeam;
use crate::intent_admission::WorthQueryIntentEligibilityTraceEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryIntentAdmissionPreDecisionPosture {
    Admitted,
    Deferred {
        stage: &'static str,
        message: &'static str,
    },
    Violation {
        stage: &'static str,
        message: &'static str,
    },
}

impl WorthQueryIntentAdmissionPreDecisionPosture {
    pub fn stage(self) -> &'static str {
        match self {
            Self::Admitted => "eligibility-admitted",
            Self::Deferred { stage, .. } | Self::Violation { stage, .. } => stage,
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Admitted => "eligibility admitted",
            Self::Deferred { message, .. } | Self::Violation { message, .. } => message,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryIntentAdmissionEligibility {
    request: WorthQueryRawIntentAdmissionRequest,
    support_posture: WorthQueryIntentAdmissionSupportEligibility,
    capability_posture: WorthQueryIntentAdmissionCapabilityEligibility,
    policy_posture: WorthQueryIntentAdmissionPolicyEligibility,
    basis_posture: WorthQueryIntentAdmissionBasisEligibility,
    invariant_posture: WorthQueryIntentAdmissionInvariantEligibility,
    projection_source_posture: WorthQueryIntentAdmissionProjectionSourceEligibility,
    routing_support_posture: WorthQueryIntentAdmissionRoutingSupportEligibility,
    source_lane_posture: WorthQueryIntentAdmissionSourceLaneEligibility,
    authority_lane_posture: WorthQueryIntentAdmissionAuthorityLaneEligibility,
    pre_decision_posture: WorthQueryIntentAdmissionPreDecisionPosture,
    eligibility_digest: String,
}

impl WorthQueryIntentAdmissionEligibility {
    pub(crate) fn from_request(
        request: WorthQueryRawIntentAdmissionRequest,
    ) -> Result<Self, IdentityDerivationDenial> {
        let (
            support_posture,
            capability_posture,
            policy_posture,
            basis_posture,
            invariant_posture,
            projection_source_posture,
            routing_support_posture,
            source_lane_posture,
            authority_lane_posture,
            pre_decision_posture,
        ) = resolve_eligibility_facts(&request)?;
        let eligibility_digest = hash_parts(&[
            "worth_query_intent_admission_eligibility_v2".to_string(),
            format!("request:{}", request.request_digest()),
            format!("family:{}", request.family().as_str()),
            format!("entrypoint:{}", request.entrypoint().as_str()),
            format!("support:{}", support_posture.as_str()),
            format!(
                "support-detail:{}",
                support_posture.detail().unwrap_or("none")
            ),
            format!("capability:{}", capability_posture.as_str()),
            format!(
                "capability-detail:{}",
                capability_posture
                    .violation_detail()
                    .map(|(_, detail)| detail)
                    .unwrap_or("none")
            ),
            format!("policy:{}", policy_posture.as_str()),
            format!(
                "policy-detail:{}",
                policy_posture.detail().unwrap_or("none")
            ),
            format!("basis:{}", basis_posture.as_str()),
            format!("basis-detail:{}", basis_posture.detail().unwrap_or("none")),
            format!("invariant:{}", invariant_posture.as_str()),
            format!(
                "invariant-detail:{}",
                invariant_posture.detail().unwrap_or("none")
            ),
            format!("projection-source:{}", projection_source_posture.as_str()),
            format!(
                "projection-source-detail:{}",
                projection_source_posture.detail().unwrap_or("none")
            ),
            format!("routing-support:{}", routing_support_posture.as_str()),
            format!(
                "routing-support-detail:{}",
                routing_support_posture.detail().unwrap_or("none")
            ),
            format!("source-lane:{}", source_lane_posture.as_str()),
            format!(
                "source-lane-detail:{}",
                source_lane_posture.detail().as_deref().unwrap_or("none")
            ),
            format!("authority-lane:{}", authority_lane_posture.as_str()),
            format!(
                "authority-lane-detail:{}",
                authority_lane_posture.detail().as_deref().unwrap_or("none")
            ),
            format!("pre-decision:{}", pre_decision_posture.stage()),
            format!("pre-decision-message:{}", pre_decision_posture.message()),
        ]);
        Ok(Self {
            request,
            support_posture,
            capability_posture,
            policy_posture,
            basis_posture,
            invariant_posture,
            projection_source_posture,
            routing_support_posture,
            source_lane_posture,
            authority_lane_posture,
            pre_decision_posture,
            eligibility_digest,
        })
    }

    pub fn request(&self) -> &WorthQueryRawIntentAdmissionRequest {
        &self.request
    }

    pub fn support_posture(&self) -> WorthQueryIntentAdmissionSupportEligibility {
        self.support_posture
    }

    pub fn capability_posture(&self) -> WorthQueryIntentAdmissionCapabilityEligibility {
        self.capability_posture
    }

    pub fn policy_posture(&self) -> WorthQueryIntentAdmissionPolicyEligibility {
        self.policy_posture
    }

    pub fn basis_posture(&self) -> WorthQueryIntentAdmissionBasisEligibility {
        self.basis_posture
    }

    pub fn invariant_posture(&self) -> WorthQueryIntentAdmissionInvariantEligibility {
        self.invariant_posture
    }

    pub fn projection_source_posture(
        &self,
    ) -> WorthQueryIntentAdmissionProjectionSourceEligibility {
        self.projection_source_posture
    }

    pub fn routing_support_posture(&self) -> WorthQueryIntentAdmissionRoutingSupportEligibility {
        self.routing_support_posture
    }

    pub fn admitted_execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.routing_support_posture.covered_execution_seam()
    }

    pub fn source_lane_posture(&self) -> WorthQueryIntentAdmissionSourceLaneEligibility {
        self.source_lane_posture
    }

    pub fn authority_lane_posture(&self) -> WorthQueryIntentAdmissionAuthorityLaneEligibility {
        self.authority_lane_posture
    }

    pub fn pre_decision_posture(&self) -> WorthQueryIntentAdmissionPreDecisionPosture {
        self.pre_decision_posture
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.eligibility_digest
    }

    pub fn trace_evidence(&self) -> WorthQueryIntentEligibilityTraceEvidence {
        WorthQueryIntentEligibilityTraceEvidence::new(
            self.support_posture,
            self.capability_posture,
            self.policy_posture,
            self.basis_posture,
            self.invariant_posture,
            self.projection_source_posture,
            self.routing_support_posture,
            self.source_lane_posture,
            self.authority_lane_posture,
            self.request
                .runtime_declaration()
                .and_then(|declaration| declaration.effect_trigger().cloned()),
            self.eligibility_digest.clone(),
        )
    }
}
