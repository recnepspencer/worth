use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::{WorthQueryAdmittedIntentPlan, WorthQueryIntentDeclaration};

use super::bind_requested;
use crate::domain_capabilities::payloads::{
    WorthQueryContinuityContributionPayload, WorthQueryContinuityContributionPosture,
    WorthQueryContinuityRuntimeSemantics,
};
use crate::domain_capabilities::proof_integration::WorthQueryRequestedContinuityContribution;
use crate::domain_capabilities::targets::{
    WorthQueryAdmittedPlanBoundContributionTarget, WorthQueryDeclarationBoundContributionTarget,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryContinuityContributionAuthoring {
    pub(super) payload: WorthQueryContinuityContributionPayload,
}

impl WorthQueryContinuityContributionAuthoring {
    pub fn preserved(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryContinuityContributionPosture::Preserved,
            semantic_code,
            detail,
        )
    }

    pub fn preserved_rebind(
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identity: impl Into<String>,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryContinuityContributionPosture::Preserved,
            semantic_code,
            detail,
            WorthQueryContinuityRuntimeSemantics::new(
                crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget,
                crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSingleSuccessor,
                prior_authoritative_identity,
                [successor_authoritative_identity],
            )?,
        )
    }

    pub fn split(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryContinuityContributionPosture::Split,
            semantic_code,
            detail,
        )
    }

    pub fn split_successors<I, S>(
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identities: I,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::with_runtime_semantics(
            WorthQueryContinuityContributionPosture::Split,
            semantic_code,
            detail,
            WorthQueryContinuityRuntimeSemantics::new(
                crate::runtime::WorthQueryContinuityMutationFamily::SplitExistingTarget,
                crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesAsSplitSuccessors,
                prior_authoritative_identity,
                successor_authoritative_identities,
            )?,
        )
    }

    pub fn replaced(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryContinuityContributionPosture::Replaced,
            semantic_code,
            detail,
        )
    }

    pub fn replaced_merge_successor(
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identity: impl Into<String>,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::with_runtime_semantics(
            WorthQueryContinuityContributionPosture::Replaced,
            semantic_code,
            detail,
            WorthQueryContinuityRuntimeSemantics::new(
                crate::runtime::WorthQueryContinuityMutationFamily::RebindExistingTarget,
                crate::runtime::WorthQueryContinuityMutationOutcomeClass::ContinuesViaTruthLoweredCanonicalMergeSuccessor,
                prior_authoritative_identity,
                [successor_authoritative_identity],
            )?,
        )
    }

    pub fn correspondence_only(
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::new(
            WorthQueryContinuityContributionPosture::CorrespondenceOnly,
            semantic_code,
            detail,
        )
    }

    pub fn for_admitted_intent_plan(
        self,
        plan: &WorthQueryAdmittedIntentPlan,
    ) -> Result<
        WorthQueryRequestedContinuityContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        self.bind_to_admitted_plan_target(
            WorthQueryAdmittedPlanBoundContributionTarget::for_admitted_intent_plan(plan),
        )
    }

    pub fn for_intent_declaration(
        self,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<
        WorthQueryRequestedContinuityContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        self.bind_to_declaration_target(
            WorthQueryDeclarationBoundContributionTarget::for_intent_declaration(declaration),
        )
    }

    pub fn bind_to_admitted_plan_target(
        self,
        target: WorthQueryAdmittedPlanBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedContinuityContribution<WorthQueryAdmittedPlanBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub fn bind_to_declaration_target(
        self,
        target: WorthQueryDeclarationBoundContributionTarget,
    ) -> Result<
        WorthQueryRequestedContinuityContribution<WorthQueryDeclarationBoundContributionTarget>,
        IdentityDerivationDenial,
    > {
        bind_requested(self.payload, target)
    }

    pub(crate) fn bind_to_installed_target<T>(
        self,
        target: crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
    ) -> Result<
        WorthQueryRequestedContinuityContribution<
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >,
        IdentityDerivationDenial,
    >
    where
        T: crate::domain_capabilities::WorthQueryDomainCapabilityTargetBinding,
        (WorthQueryContinuityContributionPayload, T):
            crate::domain_capabilities::proof_integration::AllowedContributionBinding<
                WorthQueryContinuityContributionPayload,
                T,
            >,
    {
        bind_requested::<
            WorthQueryContinuityContributionPayload,
            crate::domain_capabilities::WorthQueryInstalledDomainContributionTarget<T>,
        >(self.payload, target)
    }

    fn new(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryContinuityContributionPayload::new(posture, semantic_code, detail)?,
        })
    }

    fn with_runtime_semantics(
        posture: WorthQueryContinuityContributionPosture,
        semantic_code: impl Into<String>,
        detail: impl Into<String>,
        runtime_semantics: WorthQueryContinuityRuntimeSemantics,
    ) -> Result<Self, IdentityDerivationDenial> {
        Ok(Self {
            payload: WorthQueryContinuityContributionPayload::with_runtime_semantics(
                posture,
                semantic_code,
                detail,
                Some(runtime_semantics),
            )?,
        })
    }
}
