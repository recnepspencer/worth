use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_runtime_bridge::facade::BridgeCausalExplanationEnvelope;

use crate::runtime::{
    CausalEvidenceFamily, CausalEvidenceReferenceSet, CausalInspectionExplanationFamily,
    CausalInspectionMaterializationPolicy, CausalInspectionRedactionPolicy,
    CausalInspectionRichness, CausalInspectionTarget, WorthQueryAdmittedIntentPlan,
    WorthQueryIntentDeclaration, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryEnvelopeSource,
};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQueryExplanationContributionPayload, WorthQueryExplanationContributionPosture,
    WorthQueryExplanationRuntimeSemantics,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedExplanationContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
    WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExplanationContributionAuthoring {
    payload: WorthQueryExplanationContributionPayload,
}

impl WorthQueryExplanationContributionAuthoring {
    pub fn requires_context(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryExplanationContributionPosture::RequiresContext,
            semantic_code,
            detail,
        )
    }

    pub fn explains_fallback(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryExplanationContributionPosture::ExplainsFallback,
            semantic_code,
            detail,
        )
    }

    pub fn explains_ambiguity(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryExplanationContributionPosture::ExplainsAmbiguity,
            semantic_code,
            detail,
        )
    }

    pub fn cross_runtime_causal_context(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        reference_set: CausalEvidenceReferenceSet,
        target: CausalInspectionTarget,
        requested_evidence_families: Vec<CausalEvidenceFamily>,
        bridge_envelope: BridgeCausalExplanationEnvelope,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryExplanationContributionPosture::RequiresContext,
            semantic_code,
            detail,
            WorthQueryExplanationRuntimeSemantics::new(
                reference_set,
                target,
                CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
                CausalInspectionRichness::ReferenceOnly,
                requested_evidence_families,
                Some(bridge_envelope),
                redaction_policy,
                materialization_policy,
            )?,
        )
    }

    pub fn cross_runtime_fallback_explanation(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        reference_set: CausalEvidenceReferenceSet,
        target: CausalInspectionTarget,
        requested_evidence_families: Vec<CausalEvidenceFamily>,
        bridge_envelope: BridgeCausalExplanationEnvelope,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryExplanationContributionPosture::ExplainsFallback,
            semantic_code,
            detail,
            WorthQueryExplanationRuntimeSemantics::new(
                reference_set,
                target,
                CausalInspectionExplanationFamily::CrossRuntimeCausalExplanation,
                CausalInspectionRichness::MaterializedDetail,
                requested_evidence_families,
                Some(bridge_envelope),
                redaction_policy,
                materialization_policy,
            )?,
        )
    }

    pub fn store_backed_replay_gap_explanation(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        reference_set: CausalEvidenceReferenceSet,
        target: CausalInspectionTarget,
        requested_evidence_families: Vec<CausalEvidenceFamily>,
        redaction_policy: CausalInspectionRedactionPolicy,
        materialization_policy: CausalInspectionMaterializationPolicy,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryExplanationContributionPosture::ExplainsAmbiguity,
            semantic_code,
            detail,
            WorthQueryExplanationRuntimeSemantics::new(
                reference_set,
                target,
                CausalInspectionExplanationFamily::StoreBackedReplayReconstruction,
                CausalInspectionRichness::ReferenceOnly,
                requested_evidence_families,
                None,
                redaction_policy,
                materialization_policy,
            )?,
        )
    }

    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<
        WorthQueryRequestedExplanationContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        self.bind_to_declaration_target(
            WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(declaration),
        )
    }

    pub fn for_admitted_intent_plan(
        self,
        plan: &WorthQueryAdmittedIntentPlan,
    ) -> Result<
        WorthQueryRequestedExplanationContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
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
        WorthQueryRequestedExplanationContribution<
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
        WorthQueryRequestedExplanationContribution<
            WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
        >,
        IdentityDerivationDenial,
    >
    where
        S: WorthQueryLowerRuntimeBoundaryEnvelopeSource + ?Sized,
    {
        self.for_lower_runtime_boundary_envelope(source.lower_runtime_boundary_envelope())
    }

    pub fn bind_to_declaration_target(
        self,
        target: WorthQueryDeclarationBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedExplanationContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedExplanationContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub fn bind_to_lower_runtime_boundary_target(
        self,
        target: WorthQueryLowerRuntimeBoundaryBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedExplanationContribution<
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
        WorthQueryRequestedExplanationContribution<
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >,
        IdentityDerivationDenial,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryExplanationContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQueryExplanationContributionPayload,
                T,
            >,
    {
        bind_requested::<
            WorthQueryExplanationContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)
    }

    fn new(
        posture: WorthQueryExplanationContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryExplanationContributionPayload::new(posture, semantic_code, detail)?,
        })
    }

    fn with_runtime_semantics(
        posture: WorthQueryExplanationContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: WorthQueryExplanationRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryExplanationContributionPayload::with_runtime_semantics(
                posture,
                semantic_code,
                detail,
                Some(runtime_semantics),
            )?,
        })
    }
}
