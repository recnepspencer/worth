use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
mod construction;

use crate::basis_lifecycle::{
    scope_basis_for_observation, AdmittedBasisCapability, ObservationLaneWitness,
    ScopedObservationBasis,
};
use crate::identity::hash_parts;
use crate::projection_consumption::{
    AdmittedProjectionConsumption, MaterializedProjectionContract, ProjectionConsumptionWarnings,
};
use crate::runtime::WorthQueryIntentDeclaration;

use super::{
    inspection::{
        WorthQueryDerivedInspectionExecutionPlan, WorthQueryDerivedMaterializationExecutionPlan,
    },
    mutation::{
        WorthQueryAuthoritativeMutationBatchExecutionPlan,
        WorthQueryAuthoritativeMutationExecutionPlan,
    },
    read::{WorthQueryLiveReadExecutionPlan, WorthQueryReadExecutionPlan},
    routing::WorthQueryExistingTruthProbeExecutionPlan,
    unified_inspection::WorthQueryUnifiedInspectionExecutionPlan,
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionEligibility,
    WorthQueryIntentAdmissionExecutionSeam, WorthQueryIntentAdmissionFamily,
    WorthQueryIntentEligibilityTraceEvidence,
};

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryAuthoritativeIntentExecutionPlan {
    inner: WorthQueryAdmittedIntentPlanCore,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryEffectTriggeredIntentExecutionPlan {
    inner: WorthQueryAdmittedIntentPlanCore,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryBasisObservationPlan {
    inner: WorthQueryAdmittedIntentPlanCore,
    capability: AdmittedBasisCapability<ObservationLaneWitness>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryProjectionConsumptionPlan {
    inner: WorthQueryAdmittedIntentPlanCore,
    admitted: AdmittedProjectionConsumption,
    warnings: Option<ProjectionConsumptionWarnings>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WorthQueryAdmittedIntentPlan {
    Authoritative(WorthQueryAuthoritativeIntentExecutionPlan),
    EffectTriggered(WorthQueryEffectTriggeredIntentExecutionPlan),
    AuthoritativeMutation(WorthQueryAuthoritativeMutationExecutionPlan),
    AuthoritativeMutationBatch(WorthQueryAuthoritativeMutationBatchExecutionPlan),
    ReadExecution(WorthQueryReadExecutionPlan),
    LiveReadExecution(WorthQueryLiveReadExecutionPlan),
    DerivedMaterialization(WorthQueryDerivedMaterializationExecutionPlan),
    DerivedInspection(WorthQueryDerivedInspectionExecutionPlan),
    UnifiedInspection(WorthQueryUnifiedInspectionExecutionPlan),
    ExistingTruthProbeRouting(WorthQueryExistingTruthProbeExecutionPlan),
    BasisObservation(WorthQueryBasisObservationPlan),
    ProjectionConsumption(WorthQueryProjectionConsumptionPlan),
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorthQueryAdmittedIntentPlanCore {
    pub(super) family: WorthQueryIntentAdmissionFamily,
    pub(super) entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
    pub(super) execution_seam: Option<WorthQueryIntentAdmissionExecutionSeam>,
    pub(super) request_digest: String,
    pub(super) eligibility_digest: String,
    pub(super) eligibility_trace: WorthQueryIntentEligibilityTraceEvidence,
    pub(super) intent_name: String,
    pub(super) declaration: Option<WorthQueryIntentDeclaration>,
    pub(super) decision_digest: String,
}

impl WorthQueryAuthoritativeIntentExecutionPlan {
    pub(crate) fn core(&self) -> &WorthQueryAdmittedIntentPlanCore {
        &self.inner
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.inner.family
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.inner.entrypoint
    }

    pub fn execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.inner.execution_seam
    }

    pub fn declaration(&self) -> &WorthQueryIntentDeclaration {
        self.inner.runtime_declaration()
    }

    pub fn request_digest(&self) -> &str {
        &self.inner.request_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.inner.eligibility_digest
    }

    pub fn eligibility_trace(&self) -> &WorthQueryIntentEligibilityTraceEvidence {
        &self.inner.eligibility_trace
    }

    pub fn decision_digest(&self) -> &str {
        &self.inner.decision_digest
    }
}

impl WorthQueryEffectTriggeredIntentExecutionPlan {
    pub(crate) fn core(&self) -> &WorthQueryAdmittedIntentPlanCore {
        &self.inner
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.inner.family
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.inner.entrypoint
    }

    pub fn execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.inner.execution_seam
    }

    pub fn declaration(&self) -> &WorthQueryIntentDeclaration {
        self.inner.runtime_declaration()
    }

    pub fn request_digest(&self) -> &str {
        &self.inner.request_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.inner.eligibility_digest
    }

    pub fn eligibility_trace(&self) -> &WorthQueryIntentEligibilityTraceEvidence {
        &self.inner.eligibility_trace
    }

    pub fn decision_digest(&self) -> &str {
        &self.inner.decision_digest
    }
}

impl WorthQueryBasisObservationPlan {
    pub fn execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.inner.execution_seam
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.inner.family
    }

    pub fn request_digest(&self) -> &str {
        &self.inner.request_digest
    }

    pub fn scope(self) -> ScopedObservationBasis {
        scope_basis_for_observation(self.capability)
    }
}

impl WorthQueryProjectionConsumptionPlan {
    pub fn execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.inner.execution_seam
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.inner.family
    }

    pub fn request_digest(&self) -> &str {
        &self.inner.request_digest
    }

    pub fn bind_contract(
        &self,
    ) -> Result<MaterializedProjectionContract, IdentityDerivationDenial> {
        self.admitted.bind_contract()
    }

    pub fn warning_kinds(&self) -> Option<&ProjectionConsumptionWarnings> {
        self.warnings.as_ref()
    }
}

impl WorthQueryAdmittedIntentPlan {
    fn core(&self) -> &WorthQueryAdmittedIntentPlanCore {
        match self {
            Self::Authoritative(plan) => plan.core(),
            Self::EffectTriggered(plan) => plan.core(),
            Self::AuthoritativeMutation(plan) => plan.core(),
            Self::AuthoritativeMutationBatch(plan) => plan.core(),
            Self::ReadExecution(plan) => plan.core(),
            Self::LiveReadExecution(plan) => plan.core(),
            Self::DerivedMaterialization(plan) => &plan.inner,
            Self::DerivedInspection(plan) => &plan.inner,
            Self::UnifiedInspection(plan) => &plan.inner,
            Self::ExistingTruthProbeRouting(plan) => plan.core(),
            Self::BasisObservation(plan) => &plan.inner,
            Self::ProjectionConsumption(plan) => &plan.inner,
        }
    }

    pub fn family(&self) -> WorthQueryIntentAdmissionFamily {
        self.core().family
    }

    pub fn entrypoint(&self) -> WorthQueryIntentAdmissionCoveredEntrypoint {
        self.core().entrypoint
    }

    pub fn execution_seam(&self) -> Option<WorthQueryIntentAdmissionExecutionSeam> {
        self.core().execution_seam
    }

    pub fn request_digest(&self) -> &str {
        &self.core().request_digest
    }

    pub fn eligibility_digest(&self) -> &str {
        &self.core().eligibility_digest
    }

    pub fn eligibility_trace(&self) -> &WorthQueryIntentEligibilityTraceEvidence {
        &self.core().eligibility_trace
    }

    pub fn decision_digest(&self) -> &str {
        &self.core().decision_digest
    }
}

impl WorthQueryAdmittedIntentPlanCore {
    pub(super) fn from_eligibility(
        eligibility: WorthQueryIntentAdmissionEligibility,
        execution_seam: Option<WorthQueryIntentAdmissionExecutionSeam>,
    ) -> Self {
        let decision_digest = hash_parts(&[
            "worth_query_intent_admission_decision_v1".to_string(),
            format!("family:{}", eligibility.request().family().as_str()),
            format!("entrypoint:{}", eligibility.request().entrypoint().as_str()),
            format!("request:{}", eligibility.request().request_digest()),
            format!("eligibility:{}", eligibility.eligibility_digest()),
            "decision:admitted".to_string(),
        ]);
        Self {
            family: eligibility.request().family(),
            entrypoint: eligibility.request().entrypoint(),
            execution_seam,
            request_digest: eligibility.request().request_digest().to_string(),
            eligibility_digest: eligibility.eligibility_digest().to_string(),
            eligibility_trace: eligibility.trace_evidence(),
            intent_name: eligibility.request().intent_name().to_string(),
            declaration: eligibility.request().runtime_declaration().cloned(),
            decision_digest,
        }
    }

    fn runtime_declaration(&self) -> &WorthQueryIntentDeclaration {
        self.declaration.as_ref().unwrap_or_else(|| {
            panic!(
                "runtime declaration requested from non-runtime admitted plan: {}",
                self.intent_name
            )
        })
    }
}
