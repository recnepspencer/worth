use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{
    WorthQueryAdmittedIntentPlan, WorthQueryIntentDeclaration,
    WorthQueryLowerRuntimeBoundaryEnvelope, WorthQueryLowerRuntimeBoundaryEnvelopeSource,
};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQuerySupportContributionPayload, WorthQuerySupportContributionPosture,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedSupportContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQuerySupportContributionAuthoring {
    payload: WorthQuerySupportContributionPayload,
}

impl WorthQuerySupportContributionAuthoring {
    pub fn declaration_support(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQuerySupportContributionPosture::DeclarationSupport,
            semantic_code,
            detail,
        )
    }

    pub fn declaration_traceability(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQuerySupportContributionPosture::DeclarationTraceability,
            semantic_code,
            detail,
        )
    }

    pub fn narrowed_support(semantic_code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::new(
            WorthQuerySupportContributionPosture::NarrowedSupport,
            semantic_code,
            detail,
        )
    }

    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<
        WorthQueryRequestedSupportContribution<WorthQueryDeclarationBoundContributionTarget>,
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
        WorthQueryRequestedSupportContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(self.bind_to_admitted_plan_target(
            WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(plan),
        )?)
    }

    pub fn for_lower_runtime_boundary_envelope(
        self,
        envelope: &WorthQueryLowerRuntimeBoundaryEnvelope,
    ) -> Result<
        WorthQueryRequestedSupportContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        Ok(self.bind_to_lower_runtime_boundary_target(
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget::for_lower_runtime_boundary_envelope(
                envelope,
            ),
        )?)
    }

    pub fn for_lower_runtime_boundary_source<S>(
        self,
        source: &S,
    ) -> Result<
        WorthQueryRequestedSupportContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    >
    where
        S: WorthQueryLowerRuntimeBoundaryEnvelopeSource + ?Sized,
    {
        Ok(self.for_lower_runtime_boundary_envelope(source.lower_runtime_boundary_envelope())?)
    }

    pub fn bind_to_declaration_target(
        self,
        target: WorthQueryDeclarationBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedSupportContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedSupportContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub fn bind_to_lower_runtime_boundary_target(
        self,
        target: WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedSupportContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        Ok(bind_requested(self.payload, target)?)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> Result<
        WorthQueryRequestedSupportContribution<
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >,
        IdentityDerivationDenial,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQuerySupportContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQuerySupportContributionPayload,
                T,
            >,
    {
        Ok(bind_requested::<
            WorthQuerySupportContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)?)
    }

    fn new(
        posture: WorthQuerySupportContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            payload: WorthQuerySupportContributionPayload::new(posture, semantic_code, detail),
        }
    }
}
