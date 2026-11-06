use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::identity::hash_parts;

use super::{
    WorthQueryAdmittedIntentPlan, WorthQueryIntentAdmissionCoveredEntrypoint,
    WorthQueryIntentAdmissionEligibility, WorthQueryIntentAdmissionPreDecisionPosture,
    WorthQueryIntentAdvisoryStop, WorthQueryIntentNonAdmittedStop, WorthQueryIntentViolationStop,
    WorthQueryRawIntentAdmissionRequest,
};

pub(crate) const INTENT_ADMISSION_DECISIONS_MODULE_ROOT: &str = "intent_admission/decisions/mod.rs";
pub(crate) const INTENT_ADMISSION_DECISIONS_CHILD_MODULES: &[&str] = &[];
pub(crate) const INTENT_ADMISSION_DECISIONS_EXPORTED_SURFACE: &[&str] = &[
    "admit_runtime_intent_request",
    "WorthQueryIntentAdmissionDecision",
    "WorthQueryIntentAdvisoryDecision",
    "WorthQueryIntentViolationDecision",
];

pub(crate) fn admit_runtime_intent_request(
    request: WorthQueryRawIntentAdmissionRequest,
) -> Result<WorthQueryIntentAdmissionDecision, IdentityDerivationDenial> {
    let eligibility = WorthQueryIntentAdmissionEligibility::from_request(request)?;
    Ok(match eligibility.pre_decision_posture() {
        WorthQueryIntentAdmissionPreDecisionPosture::Admitted => {
            WorthQueryIntentAdmissionDecision::Admitted(
                WorthQueryAdmittedIntentPlan::from_eligibility(eligibility)?,
            )
        }
        WorthQueryIntentAdmissionPreDecisionPosture::Deferred { stage, message } => {
            WorthQueryIntentAdmissionDecision::Advisory(WorthQueryIntentAdvisoryDecision::new(
                eligibility.request().family(),
                eligibility.request().entrypoint(),
                stage,
                message,
                eligibility.request().request_digest(),
                eligibility.eligibility_digest(),
            ))
        }
        WorthQueryIntentAdmissionPreDecisionPosture::Violation { stage, message } => {
            WorthQueryIntentAdmissionDecision::Violation(
                WorthQueryIntentViolationDecision::from_eligibility_violation(
                    &eligibility,
                    stage,
                    message,
                ),
            )
        }
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryIntentAdmissionDecision {
    Admitted(WorthQueryAdmittedIntentPlan),
    Advisory(WorthQueryIntentAdvisoryDecision),
    Violation(WorthQueryIntentViolationDecision),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdvisoryDecision {
    family: super::WorthQueryIntentAdmissionFamily,
    entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
    stage: &'static str,
    message: String,
    request_digest: String,
    eligibility_digest: String,
    decision_digest: String,
}

impl WorthQueryIntentAdvisoryDecision {
    pub(crate) fn new(
        family: super::WorthQueryIntentAdmissionFamily,
        entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
        stage: &'static str,
        message: impl Into<String>,
        request_digest: &str,
        eligibility_digest: &str,
    ) -> Self {
        let message = message.into();
        let decision_digest = hash_parts(&[
            "worth_query_intent_advisory_decision_v1".to_string(),
            format!("family:{}", family.as_str()),
            format!("entrypoint:{}", entrypoint.as_str()),
            format!("stage:{stage}"),
            format!("message:{message}"),
            format!("request:{request_digest}"),
            format!("eligibility:{eligibility_digest}"),
        ]);
        Self {
            family,
            entrypoint,
            stage,
            message,
            request_digest: request_digest.to_string(),
            eligibility_digest: eligibility_digest.to_string(),
            decision_digest,
        }
    }

    pub fn family(&self) -> super::WorthQueryIntentAdmissionFamily {
        self.family
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.entrypoint
    }

    pub fn stage(&self) -> &'static str {
        self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.eligibility_digest
    }

    pub fn decision_digest(&self) -> &str {
        &self.decision_digest
    }

    pub(crate) fn into_violation(self) -> WorthQueryIntentViolationDecision {
        WorthQueryIntentViolationDecision::new(
            self.family(),
            self.entrypoint(),
            self.stage(),
            self.message(),
            self.request_digest(),
            self.eligibility_digest(),
        )
    }

    pub fn into_stop(self) -> WorthQueryIntentAdvisoryStop {
        WorthQueryIntentAdvisoryStop::from_decision(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentViolationDecision {
    family: super::WorthQueryIntentAdmissionFamily,
    entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
    stage: &'static str,
    message: String,
    request_digest: String,
    eligibility_digest: String,
    decision_digest: String,
}

impl WorthQueryIntentViolationDecision {
    pub(crate) fn new(
        family: super::WorthQueryIntentAdmissionFamily,
        entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
        stage: &'static str,
        message: impl Into<String>,
        request_digest: &str,
        eligibility_digest: &str,
    ) -> Self {
        let message = message.into();
        let decision_digest = hash_parts(&[
            "worth_query_intent_violation_decision_v1".to_string(),
            format!("family:{}", family.as_str()),
            format!("entrypoint:{}", entrypoint.as_str()),
            format!("stage:{stage}"),
            format!("message:{message}"),
            format!("request:{request_digest}"),
            format!("eligibility:{eligibility_digest}"),
        ]);
        Self {
            family,
            entrypoint,
            stage,
            message,
            request_digest: request_digest.to_string(),
            eligibility_digest: eligibility_digest.to_string(),
            decision_digest,
        }
    }

    pub(crate) fn from_eligibility_denial(
        eligibility: &WorthQueryIntentAdmissionEligibility,
        stage: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self::new(
            eligibility.request().family(),
            eligibility.request().entrypoint(),
            stage,
            message,
            eligibility.request().request_digest(),
            eligibility.eligibility_digest(),
        )
    }

    pub(crate) fn from_eligibility_violation(
        eligibility: &WorthQueryIntentAdmissionEligibility,
        stage: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self::from_eligibility_denial(eligibility, stage, message)
    }

    pub fn family(&self) -> super::WorthQueryIntentAdmissionFamily {
        self.family
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.entrypoint
    }

    pub fn stage(&self) -> &'static str {
        self.stage
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn request_digest(&self) -> &str {
        &self.request_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.eligibility_digest
    }

    pub fn decision_digest(&self) -> &str {
        &self.decision_digest
    }

    pub fn into_stop(self) -> WorthQueryIntentViolationStop {
        WorthQueryIntentViolationStop::from_decision(self)
    }
}

impl WorthQueryIntentAdmissionDecision {
    pub fn into_non_admitted_stop(self) -> Option<WorthQueryIntentNonAdmittedStop> {
        match self {
            Self::Admitted(_) => None,
            Self::Advisory(advisory) => Some(WorthQueryIntentNonAdmittedStop::Advisory(
                advisory.into_stop(),
            )),
            Self::Violation(violation) => Some(WorthQueryIntentNonAdmittedStop::Violation(
                violation.into_stop(),
            )),
        }
    }
}
