use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionAdmissionDiagnosticStage {
    RuntimeBackedAdmission,
    AdmissionBudget,
    DurableReloadOverclaim,
    ActiveLifecycleAllocation,
    ActivationReadiness,
}

impl QuerySubscriptionAdmissionDiagnosticStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RuntimeBackedAdmission => "runtime_backed_admission",
            Self::AdmissionBudget => "admission_budget",
            Self::DurableReloadOverclaim => "durable_reload_overclaim",
            Self::ActiveLifecycleAllocation => "active_lifecycle_allocation",
            Self::ActivationReadiness => "activation_readiness",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuerySubscriptionAdmissionDiagnosticOutcome {
    Admitted,
    Denied,
}

impl QuerySubscriptionAdmissionDiagnosticOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Admitted => "admitted",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuerySubscriptionAdmissionDiagnostics {
    stage: QuerySubscriptionAdmissionDiagnosticStage,
    outcome: QuerySubscriptionAdmissionDiagnosticOutcome,
    reason: String,
    diagnostics_identity: WorthQueryEvidenceIdentity,
}

impl QuerySubscriptionAdmissionDiagnostics {
    pub(super) fn new(
        stage: QuerySubscriptionAdmissionDiagnosticStage,
        outcome: QuerySubscriptionAdmissionDiagnosticOutcome,
        reason: impl Into<String>,
        source_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let reason = reason.into();
        let diagnostics_identity = WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::SubscriptionActivationReceipt,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_admission_diagnostics_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("stage"), stage.as_str())
        .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome.as_str())
        .field_shape(WorthQueryEvidenceTag::new("reason"), &reason)
        .field_evidence_identity(WorthQueryEvidenceTag::new("source"), source_identity)
        .seal()?;
        Ok(Self {
            stage,
            outcome,
            reason,
            diagnostics_identity,
        })
    }

    pub fn stage(&self) -> &QuerySubscriptionAdmissionDiagnosticStage {
        &self.stage
    }

    pub fn outcome(&self) -> &QuerySubscriptionAdmissionDiagnosticOutcome {
        &self.outcome
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn diagnostics_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.diagnostics_identity
    }
}
