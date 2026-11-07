use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::WorkflowCounters;

use super::context_binding::{WorkflowBasisFamily, WorkflowContextBinding};
use super::declaration_model::{
    WorkflowAuthorityTargetFamily, WorkflowBudgetClass, WorkflowCostClass,
    WorkflowDeclarationFamily, WorkflowDeclarationRequest, WorkflowFreshnessPolicy,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorkflowPredictionDriftOutcome {
    EvidenceIdentityDerivationDenied,
    WithinBudget,
    ExplicitBroadeningDenied,
    ExplicitRebindRequired,
}

impl WorkflowPredictionDriftOutcome {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied => "evidence_identity_derivation_denied",
            Self::WithinBudget => "within_budget",
            Self::ExplicitBroadeningDenied => "explicit_broadening_denied",
            Self::ExplicitRebindRequired => "explicit_rebind_required",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowAdmissionFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    UnsupportedWorkflowFamily,
    UnsupportedBasisFamily,
    InvalidBasisPairing,
    PreviewReadOnlyAuthorityRequestForbidden,
    UnsupportedAuthorityTargetFamily,
    ForbiddenWorkflowBroadening,
    ExplicitRebindRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowAdmissionError {
    failure_class: WorkflowAdmissionFailureClass,
    message: &'static str,
    drift_outcome: WorkflowPredictionDriftOutcome,
    counters: WorkflowCounters,
}

impl WorkflowAdmissionError {
    pub(super) fn identity_refused(
        cause: IdentityDerivationDenial,
        mut counters: WorkflowCounters,
    ) -> Self {
        counters.workflow_denial_count = counters.workflow_denial_count.saturating_add(1);
        Self::new(
            WorkflowAdmissionFailureClass::EvidenceIdentityDerivationDenied(cause),
            "workflow binding evidence identity exceeded its digest bounds",
            WorkflowPredictionDriftOutcome::EvidenceIdentityDerivationDenied,
            counters,
        )
    }

    pub(super) fn new(
        failure_class: WorkflowAdmissionFailureClass,
        message: &'static str,
        drift_outcome: WorkflowPredictionDriftOutcome,
        counters: WorkflowCounters,
    ) -> Self {
        Self {
            failure_class,
            message,
            drift_outcome,
            counters,
        }
    }

    pub fn failure_class(&self) -> &WorkflowAdmissionFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn drift_outcome(&self) -> &WorkflowPredictionDriftOutcome {
        &self.drift_outcome
    }

    pub fn counters(&self) -> &WorkflowCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowAdmissionReport {
    pub(super) binding_identity: WorthQueryEvidenceIdentity,
    pub(super) declaration_identity: WorthQueryEvidenceIdentity,
    pub(super) declaration_family: WorkflowDeclarationFamily,
    pub(super) basis_family: WorkflowBasisFamily,
    pub(super) authority_target_family: WorkflowAuthorityTargetFamily,
    pub(super) cost_class: WorkflowCostClass,
    pub(super) budget_class: WorkflowBudgetClass,
    pub(super) freshness_policy: WorkflowFreshnessPolicy,
    pub(super) drift_outcome: WorkflowPredictionDriftOutcome,
    pub(super) counters: WorkflowCounters,
}

impl WorkflowAdmissionReport {
    pub fn binding_digest(&self) -> &str {
        self.binding_identity.as_str()
    }

    pub fn declaration_digest(&self) -> &str {
        self.declaration_identity.as_str()
    }

    pub fn binding_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.binding_identity
    }

    pub fn declaration_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.declaration_identity
    }

    pub fn declaration_family(&self) -> &WorkflowDeclarationFamily {
        &self.declaration_family
    }

    pub fn basis_family(&self) -> &WorkflowBasisFamily {
        &self.basis_family
    }

    pub fn authority_target_family(&self) -> &WorkflowAuthorityTargetFamily {
        &self.authority_target_family
    }

    pub fn cost_class(&self) -> &WorkflowCostClass {
        &self.cost_class
    }

    pub fn budget_class(&self) -> &WorkflowBudgetClass {
        &self.budget_class
    }

    pub fn freshness_policy(&self) -> &WorkflowFreshnessPolicy {
        &self.freshness_policy
    }

    pub fn drift_outcome(&self) -> &WorkflowPredictionDriftOutcome {
        &self.drift_outcome
    }

    pub fn counters(&self) -> &WorkflowCounters {
        &self.counters
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryWorkflowDeclaration {
    pub(super) binding: WorkflowContextBinding,
    pub(super) request: WorkflowDeclarationRequest,
    pub(super) report: WorkflowAdmissionReport,
}

impl QueryWorkflowDeclaration {
    pub fn binding(&self) -> &WorkflowContextBinding {
        &self.binding
    }

    pub fn request(&self) -> &WorkflowDeclarationRequest {
        &self.request
    }

    pub fn report(&self) -> &WorkflowAdmissionReport {
        &self.report
    }
}
