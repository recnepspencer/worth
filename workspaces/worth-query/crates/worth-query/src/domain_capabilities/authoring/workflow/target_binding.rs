use super::WorthQueryWorkflowContributionAuthoring;
use crate::domain_capabilities::payloads::WorthQueryWorkflowContributionPayload;
use crate::domain_capabilities::proof_integration::AllowedContributionBinding;
use crate::domain_capabilities::proof_integration::WorthQueryRequestedWorkflowContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryAdmittedIntentPlan, WorthQueryIntentDeclaration};

use super::super::bind_requested;

impl WorthQueryWorkflowContributionAuthoring {
    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<
        WorthQueryRequestedWorkflowContribution<WorthQueryDeclarationBoundContributionTarget>,
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
        WorthQueryRequestedWorkflowContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
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
        WorthQueryRequestedWorkflowContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedWorkflowContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> Result<
        WorthQueryRequestedWorkflowContribution<
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >,
        IdentityDerivationDenial,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryWorkflowContributionPayload, T):
            AllowedContributionBinding<WorthQueryWorkflowContributionPayload, T>,
    {
        Ok(bind_requested::<
            WorthQueryWorkflowContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)?)
    }
}
