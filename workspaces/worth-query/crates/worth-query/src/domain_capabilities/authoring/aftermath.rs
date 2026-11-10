use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::{
    ProjectMaterializedFacts, ProjectionConsumptionBindingContext, ProjectionConsumptionSource,
};
use crate::runtime::{
    WorthQueryAdmittedIntentPlan, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryEnvelopeSource,
};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQueryAftermathContributionPayload, WorthQueryAftermathContributionPosture,
    WorthQueryAftermathRuntimeSemantics,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedAftermathContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAftermathContributionAuthoring {
    payload: WorthQueryAftermathContributionPayload,
}

impl WorthQueryAftermathContributionAuthoring {
    pub fn establishes_fact(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryAftermathContributionPosture::EstablishesFact,
            semantic_code,
            detail,
        )
    }

    pub fn consumes_fact(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryAftermathContributionPosture::ConsumesFact,
            semantic_code,
            detail,
        )
    }

    pub fn declares_residue(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryAftermathContributionPosture::DeclaresResidue,
            semantic_code,
            detail,
        )
    }

    pub fn establishes_projection_contract(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        source: ProjectionConsumptionSource,
        binding: ProjectionConsumptionBindingContext,
        requested_facts: ProjectMaterializedFacts,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryAftermathContributionPosture::EstablishesFact,
            semantic_code,
            detail,
            WorthQueryAftermathRuntimeSemantics::new(source, binding, requested_facts),
        )
    }

    pub fn consumes_projection_contract(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        source: ProjectionConsumptionSource,
        binding: ProjectionConsumptionBindingContext,
        requested_facts: ProjectMaterializedFacts,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryAftermathContributionPosture::ConsumesFact,
            semantic_code,
            detail,
            WorthQueryAftermathRuntimeSemantics::new(source, binding, requested_facts),
        )
    }

    pub fn declares_projection_residue(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        source: ProjectionConsumptionSource,
        binding: ProjectionConsumptionBindingContext,
        requested_facts: ProjectMaterializedFacts,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryAftermathContributionPosture::DeclaresResidue,
            semantic_code,
            detail,
            WorthQueryAftermathRuntimeSemantics::new(source, binding, requested_facts),
        )
    }

    pub fn for_admitted_intent_plan(
        self,
        plan: &WorthQueryAdmittedIntentPlan,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        self.bind_to_admitted_plan_target(
            WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(plan),
        )
    }

    pub fn for_lower_runtime_boundary_envelope(
        self,
        envelope: &WorthQueryLowerRuntimeBoundaryEnvelope,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        self.bind_to_lower_runtime_boundary_target(
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget::for_lower_runtime_boundary_envelope(
                envelope,
            ),
        )
    }

    pub fn for_lower_runtime_boundary_source<S>(
        self,
        source: &S,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    >
    where
        S: WorthQueryLowerRuntimeBoundaryEnvelopeSource + ?Sized,
    {
        self.for_lower_runtime_boundary_envelope(source.lower_runtime_boundary_envelope())
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub fn bind_to_lower_runtime_boundary_target(
        self,
        target: WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> Result<
        WorthQueryRequestedAftermathContribution<
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >,
        IdentityDerivationDenial,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryAftermathContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQueryAftermathContributionPayload,
                T,
            >,
    {
        bind_requested::<
            WorthQueryAftermathContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)
    }

    fn new(
        posture: WorthQueryAftermathContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryAftermathContributionPayload::new(posture, semantic_code, detail)?,
        })
    }

    fn with_runtime_semantics(
        posture: WorthQueryAftermathContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: WorthQueryAftermathRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryAftermathContributionPayload::with_runtime_semantics(
                posture,
                semantic_code,
                detail,
                Some(runtime_semantics),
            )?,
        })
    }
}
