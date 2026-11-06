use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{
    WorthQueryIntentDeclaration, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryEnvelopeSource,
};
use worth_relational::facade::runtime::{InvariantCatalog, InvariantRegistration};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQueryGraphCapabilityRuntimeSemantics, WorthQueryGraphInvariantDenialRuntimeSemantics,
    WorthQueryInvariantCapabilityContributionPayload,
    WorthQueryInvariantCapabilityContributionPosture,
    WorthQueryInvariantRegistrationRuntimeSemantics,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedInvariantCapabilityContribution;
use crate::domain_capabilities::targets::{
    WorthQueryDeclarationBoundContributionTarget,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantCapabilityContributionAuthoring {
    payload: WorthQueryInvariantCapabilityContributionPayload,
}

impl WorthQueryInvariantCapabilityContributionAuthoring {
    pub fn capability_gap(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryInvariantCapabilityContributionPosture::CapabilityGap,
            semantic_code,
            detail,
        )?)
    }

    pub fn graph_capability_gap(
        capability_family: impl Into<String>,
        capability_class: crate::runtime::WorthQueryGraphCompositionCapabilityClass,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_graph_capability(
            WorthQueryInvariantCapabilityContributionPosture::CapabilityGap,
            semantic_code,
            detail,
            WorthQueryGraphCapabilityRuntimeSemantics::new(capability_family, capability_class),
        )?)
    }

    pub fn invariant_denial(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryInvariantCapabilityContributionPosture::InvariantDenial,
            semantic_code,
            detail,
        )?)
    }

    pub fn graph_invariant_denial(
        invariant_family: impl Into<String>,
        declared_collections: impl IntoIterator<Item = impl Into<String>>,
        declared_symbols: impl IntoIterator<Item = impl Into<String>>,
        target_combination_families: impl IntoIterator<Item = impl Into<String>>,
        lifecycle_families: impl IntoIterator<Item = impl Into<String>>,
        program_digest: impl Into<String>,
        breadth_digest: impl Into<String>,
        counter_snapshot: impl Into<String>,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_graph_invariant_denial(
            WorthQueryInvariantCapabilityContributionPosture::InvariantDenial,
            semantic_code,
            detail,
            WorthQueryGraphInvariantDenialRuntimeSemantics::new(
                invariant_family,
                declared_collections,
                declared_symbols,
                target_combination_families,
                lifecycle_families,
                program_digest,
                breadth_digest,
                counter_snapshot,
            )?,
        )?)
    }

    pub fn support_summary(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryInvariantCapabilityContributionPosture::SupportSummary,
            semantic_code,
            detail,
        )?)
    }

    pub fn graph_support_summary(
        capability_family: impl Into<String>,
        capability_class: crate::runtime::WorthQueryGraphCompositionCapabilityClass,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_graph_capability(
            WorthQueryInvariantCapabilityContributionPosture::SupportSummary,
            semantic_code,
            detail,
            WorthQueryGraphCapabilityRuntimeSemantics::new(capability_family, capability_class),
        )?)
    }

    pub fn invariant_registration(
        invariant_catalog: InvariantCatalog,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_invariant_registration(
            WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration,
            semantic_code,
            detail,
            WorthQueryInvariantRegistrationRuntimeSemantics::new(invariant_catalog)?,
        )?)
    }

    pub fn invariant_rule_registration(
        registration: InvariantRegistration,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::with_invariant_registration(
            WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration,
            semantic_code,
            detail,
            WorthQueryInvariantRegistrationRuntimeSemantics::from_registration(registration)?,
        )?)
    }

    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryDeclarationBoundContributionTarget,
    > {
        self.bind_to_declaration_target(
            WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(declaration),
        )
    }

    pub fn for_lower_runtime_boundary_envelope(
        self,
        envelope: &WorthQueryLowerRuntimeBoundaryEnvelope,
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
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
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    >
    where
        S: WorthQueryLowerRuntimeBoundaryEnvelopeSource + ?Sized,
    {
        self.for_lower_runtime_boundary_envelope(source.lower_runtime_boundary_envelope())
    }

    pub fn bind_to_declaration_target(
        self,
        target: WorthQueryDeclarationBoundContributionTarget,
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryDeclarationBoundContributionTarget,
    > {
        bind_requested(self.payload, target)
    }

    pub fn bind_to_lower_runtime_boundary_target(
        self,
        target: WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    > {
        bind_requested(self.payload, target)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> WorthQueryRequestedInvariantCapabilityContribution<
        crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryInvariantCapabilityContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQueryInvariantCapabilityContributionPayload,
                T,
            >,
    {
        bind_requested::<
            WorthQueryInvariantCapabilityContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)
    }

    fn new(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryInvariantCapabilityContributionPayload::new(
                posture,
                semantic_code,
                detail,
            )?,
        })
    }

    fn with_graph_capability(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        graph_capability: WorthQueryGraphCapabilityRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryInvariantCapabilityContributionPayload::with_graph_capability(
                posture,
                semantic_code,
                detail,
                Some(graph_capability),
            )?,
        })
    }

    fn with_graph_invariant_denial(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        graph_invariant_denial: WorthQueryGraphInvariantDenialRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryInvariantCapabilityContributionPayload::with_graph_invariant_denial(
                posture,
                semantic_code,
                detail,
                Some(graph_invariant_denial),
            )?,
        })
    }

    fn with_invariant_registration(
        posture: WorthQueryInvariantCapabilityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        invariant_registration: WorthQueryInvariantRegistrationRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryInvariantCapabilityContributionPayload::with_invariant_registration(
                posture,
                semantic_code,
                detail,
                Some(invariant_registration),
            )?,
        })
    }
}
