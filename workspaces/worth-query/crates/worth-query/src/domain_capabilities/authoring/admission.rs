use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryAdmittedIntentPlan, WorthQueryIntentDeclaration};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQueryAdmissionContributionPayload, WorthQueryAdmissionContributionPosture,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedAdmissionContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmissionContributionAuthoring {
    payload: WorthQueryAdmissionContributionPayload,
}

impl WorthQueryAdmissionContributionAuthoring {
    pub fn advisory(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryAdmissionContributionPosture::Advisory,
            semantic_code,
            detail,
        )?)
    }

    pub fn advisory_at_stage(
        decision_stage: &'static str,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_decision_stage(
            WorthQueryAdmissionContributionPosture::Advisory,
            semantic_code,
            detail,
            decision_stage,
        )?)
    }

    pub fn violation(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryAdmissionContributionPosture::Violation,
            semantic_code,
            detail,
        )?)
    }

    pub fn violation_at_stage(
        decision_stage: &'static str,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_decision_stage(
            WorthQueryAdmissionContributionPosture::Violation,
            semantic_code,
            detail,
            decision_stage,
        )?)
    }

    pub fn support_only(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryAdmissionContributionPosture::SupportOnly,
            semantic_code,
            detail,
        )?)
    }

    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<
        WorthQueryRequestedAdmissionContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(self.bind_to_declaration_target(
            WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(declaration),
        )?)
    }

    pub fn for_admitted_intent_plan(
        self,
        plan: &WorthQueryAdmittedIntentPlan,
    ) -> Result<
        WorthQueryRequestedAdmissionContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(self.bind_to_admitted_plan_target(
            WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(plan),
        )?)
    }

    pub fn bind_to_declaration_target(
        self,
        target: WorthQueryDeclarationBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedAdmissionContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedAdmissionContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> WorthQueryRequestedAdmissionContribution<
        crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryAdmissionContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQueryAdmissionContributionPayload,
                T,
            >,
    {
        bind_requested::<
            WorthQueryAdmissionContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)
    }

    fn new(
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryAdmissionContributionPayload::new(posture, semantic_code, detail)?,
        })
    }

    fn with_decision_stage(
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        decision_stage: &'static str,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryAdmissionContributionPayload::with_decision_stage(
                posture,
                semantic_code,
                detail,
                decision_stage,
            )?,
        })
    }
}
