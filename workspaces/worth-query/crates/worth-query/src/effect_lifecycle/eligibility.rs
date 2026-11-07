use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::workflow::QueryWorkflowDeclaration;

use super::counters::EffectLifecycleCounters;
use super::normalized::NormalizedEffectIntent;
use super::support_contract::EffectDeferredSupportContract;
use super::support_matrix::EffectSupportCause;
use super::taxonomy::DeniedEffectEligibilityKind;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EffectEligibilityDecisionTrace {
    normalized_identity: WorthQueryEvidenceIdentity,
    outcome: &'static str,
    message: &'static str,
    cause: &'static str,
    trace_identity: WorthQueryEvidenceIdentity,
}

impl EffectEligibilityDecisionTrace {
    pub(crate) fn new(
        normalized: &NormalizedEffectIntent,
        outcome: &'static str,
        message: &'static str,
        cause: &'static str,
    ) -> Result<Self, IdentityDerivationDenial> {
        let trace_identity =
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WorkflowMutationLowering)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "effect_eligibility_decision_trace_v1",
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("normalized"),
                    normalized.normalized_identity(),
                )
                .field_shape(WorthQueryEvidenceTag::new("outcome"), outcome)
                .field_shape(WorthQueryEvidenceTag::new("message"), message)
                .field_shape(WorthQueryEvidenceTag::new("cause"), cause)
                .seal()?;
        Ok(Self {
            normalized_identity: normalized.normalized_identity().clone(),
            outcome,
            message,
            cause,
            trace_identity,
        })
    }

    pub fn normalized_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.normalized_identity
    }

    pub fn normalized_for_reporting(&self) -> &str {
        self.normalized_identity.as_str()
    }

    pub fn outcome(&self) -> &'static str {
        self.outcome
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn cause(&self) -> &'static str {
        self.cause
    }

    pub fn trace_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.trace_identity
    }

    pub fn trace_for_reporting(&self) -> &str {
        self.trace_identity.as_str()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EffectEligibility {
    normalized: NormalizedEffectIntent,
    workflow_declaration: QueryWorkflowDeclaration,
    decision_trace: EffectEligibilityDecisionTrace,
    counters: EffectLifecycleCounters,
}

impl EffectEligibility {
    pub(crate) fn new(
        normalized: NormalizedEffectIntent,
        workflow_declaration: QueryWorkflowDeclaration,
        support_row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            decision_trace: EffectEligibilityDecisionTrace::new(
                &normalized,
                "admitted",
                "effect is admitted",
                "supported",
            )?,
            normalized,
            workflow_declaration,
            counters: EffectLifecycleCounters::admitted(support_row_count),
        })
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn decision_trace(&self) -> &EffectEligibilityDecisionTrace {
        &self.decision_trace
    }

    pub fn workflow_declaration(&self) -> &QueryWorkflowDeclaration {
        &self.workflow_declaration
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdvisoryEffectEligibility {
    normalized: NormalizedEffectIntent,
    advisory_cause: EffectSupportCause,
    decision_trace: EffectEligibilityDecisionTrace,
    counters: EffectLifecycleCounters,
}

impl AdvisoryEffectEligibility {
    pub(crate) fn new(
        normalized: NormalizedEffectIntent,
        advisory_cause: EffectSupportCause,
        message: &'static str,
        support_row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            advisory_cause,
            decision_trace: EffectEligibilityDecisionTrace::new(
                &normalized,
                "advisory",
                message,
                advisory_cause.as_str(),
            )?,
            normalized,
            counters: EffectLifecycleCounters::advisory(support_row_count),
        })
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn advisory_cause(&self) -> EffectSupportCause {
        self.advisory_cause
    }

    pub fn decision_trace(&self) -> &EffectEligibilityDecisionTrace {
        &self.decision_trace
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeniedEffectEligibility {
    normalized: NormalizedEffectIntent,
    denial_kind: DeniedEffectEligibilityKind,
    decision_trace: EffectEligibilityDecisionTrace,
    counters: EffectLifecycleCounters,
}

impl DeniedEffectEligibility {
    pub(crate) fn new(
        denial_kind: DeniedEffectEligibilityKind,
        normalized: &NormalizedEffectIntent,
        message: &'static str,
        cause: &'static str,
        support_row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            normalized: normalized.clone(),
            denial_kind,
            decision_trace: EffectEligibilityDecisionTrace::new(
                normalized, "denied", message, cause,
            )?,
            counters: EffectLifecycleCounters::denied(support_row_count),
        })
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn denial_kind(&self) -> DeniedEffectEligibilityKind {
        self.denial_kind
    }

    pub fn decision_trace(&self) -> &EffectEligibilityDecisionTrace {
        &self.decision_trace
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RebindRequiredEffectEligibility {
    normalized: NormalizedEffectIntent,
    denial_kind: DeniedEffectEligibilityKind,
    decision_trace: EffectEligibilityDecisionTrace,
    counters: EffectLifecycleCounters,
}

impl RebindRequiredEffectEligibility {
    pub(crate) fn new(
        normalized: &NormalizedEffectIntent,
        message: &'static str,
        cause: &'static str,
        support_row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            normalized: normalized.clone(),
            denial_kind: DeniedEffectEligibilityKind::PreviewRebindRequired,
            decision_trace: EffectEligibilityDecisionTrace::new(
                normalized,
                "rebind_required",
                message,
                cause,
            )?,
            counters: EffectLifecycleCounters::rebind_required(support_row_count),
        })
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn denial_kind(&self) -> DeniedEffectEligibilityKind {
        self.denial_kind
    }

    pub fn decision_trace(&self) -> &EffectEligibilityDecisionTrace {
        &self.decision_trace
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DeferredEffectEligibility {
    normalized: NormalizedEffectIntent,
    deferred_contract: EffectDeferredSupportContract,
    denial_kind: DeniedEffectEligibilityKind,
    decision_trace: EffectEligibilityDecisionTrace,
    counters: EffectLifecycleCounters,
}

impl DeferredEffectEligibility {
    pub(crate) fn new(
        deferred_contract: EffectDeferredSupportContract,
        normalized: &NormalizedEffectIntent,
        message: &'static str,
        cause: &'static str,
        support_row_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            normalized: normalized.clone(),
            denial_kind: deferred_contract.denial_kind(),
            deferred_contract,
            decision_trace: EffectEligibilityDecisionTrace::new(
                normalized, "deferred", message, cause,
            )?,
            counters: EffectLifecycleCounters::deferred(support_row_count),
        })
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn denial_kind(&self) -> DeniedEffectEligibilityKind {
        self.denial_kind
    }

    pub fn deferred_contract(&self) -> &EffectDeferredSupportContract {
        &self.deferred_contract
    }

    pub fn decision_trace(&self) -> &EffectEligibilityDecisionTrace {
        &self.decision_trace
    }

    pub fn counters(&self) -> &EffectLifecycleCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectEligibilityOutcome {
    Admitted(EffectEligibility),
    Advisory(AdvisoryEffectEligibility),
    Denied(DeniedEffectEligibility),
    RebindRequired(RebindRequiredEffectEligibility),
    Deferred(DeferredEffectEligibility),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedEffectIntent {
    normalized: NormalizedEffectIntent,
    workflow_declaration: QueryWorkflowDeclaration,
    admitted_identity: WorthQueryEvidenceIdentity,
}

impl AdmittedEffectIntent {
    pub(crate) fn new(eligibility: EffectEligibility) -> Self {
        let admitted_identity = eligibility.normalized.admitted_identity();
        Self {
            normalized: eligibility.normalized,
            workflow_declaration: eligibility.workflow_declaration,
            admitted_identity,
        }
    }

    pub fn normalized(&self) -> &NormalizedEffectIntent {
        &self.normalized
    }

    pub fn admitted_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.admitted_identity
    }

    pub fn admitted_for_reporting(&self) -> &str {
        self.admitted_identity.as_str()
    }

    pub fn workflow_declaration(&self) -> &QueryWorkflowDeclaration {
        &self.workflow_declaration
    }
}
