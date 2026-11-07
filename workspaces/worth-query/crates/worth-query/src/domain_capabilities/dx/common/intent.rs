use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_relational::facade::runtime::InvariantCatalog;

use crate::domain_capabilities::authoring::{
    WorthQueryInvariantCapabilityContributionAuthoring, WorthQuerySupportContributionAuthoring,
};
use crate::domain_capabilities::canonical_runtime::{
    materialize_intent_declaration_support_traceability_artifact,
    materialize_query_invariant_catalog_registration_artifact,
    WorthQueryIntentDeclarationSupportTraceabilityArtifact,
    WorthQueryInvariantCatalogRegistrationArtifact,
};
use crate::domain_capabilities::dx::checked::{
    WorthQueryCheckedDomainCapabilityOutcome, WorthQueryDomainCapabilityMaterializationError,
};
use crate::domain_capabilities::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use crate::domain_capabilities::payloads::{
    WorthQueryInvariantCapabilityContributionPosture, WorthQuerySupportContributionPosture,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryInstalledDeclarationContributionTarget,
};

use super::materialization::materialize_common_lane;
use super::semantic_code::qualify_semantic_code;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentDomainContributionSurface {
    pub(crate) target: WorthQueryInstalledDeclarationContributionTarget,
}

impl WorthQueryIntentDomainContributionSurface {
    pub fn supports_capability(
        self,
        semantic_code: impl Into<String>,
    ) -> WorthQueryIntentSupportDraft {
        WorthQueryIntentSupportDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQuerySupportContributionPosture::DeclarationSupport,
            semantic_code,
        )
    }

    pub fn supports_traceability(
        self,
        semantic_code: impl Into<String>,
    ) -> WorthQueryIntentSupportDraft {
        WorthQueryIntentSupportDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQuerySupportContributionPosture::DeclarationTraceability,
            semantic_code,
        )
    }

    pub fn register_invariant_catalog(
        self,
        semantic_code: impl Into<String>,
        invariant_catalog: InvariantCatalog,
    ) -> WorthQueryIntentInvariantRegistrationDraft {
        WorthQueryIntentInvariantRegistrationDraft {
            target: self.target,
            semantic_code: semantic_code.into(),
            invariant_catalog,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentSupportDraft {
    target: WorthQueryInstalledDeclarationContributionTarget,
    posture: WorthQuerySupportContributionPosture,
    semantic_code: String,
}

impl WorthQueryIntentSupportDraft {
    pub(crate) fn new(
        target: WorthQueryInstalledDeclarationContributionTarget,
        posture: WorthQuerySupportContributionPosture,
        semantic_code: impl Into<String>,
    ) -> Self {
        Self {
            target,
            posture,
            semantic_code: semantic_code.into(),
        }
    }

    pub fn because(self, detail: impl Into<String>) -> WorthQueryIntentSupportContribution {
        WorthQueryIntentSupportContribution {
            target: self.target,
            posture: self.posture,
            semantic_code: self.semantic_code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentSupportContribution {
    target: WorthQueryInstalledDeclarationContributionTarget,
    posture: WorthQuerySupportContributionPosture,
    semantic_code: String,
    detail: String,
}

impl WorthQueryIntentSupportContribution {
    pub fn try_materialize(
        self,
    ) -> WorthQueryCheckedDomainCapabilityOutcome<
        WorthQueryIntentDeclarationSupportTraceabilityArtifact,
    > {
        let semantic_code = qualify_semantic_code(self.target.authority(), &self.semantic_code);
        let target = self.target.clone();
        let requested = (match self.posture {
            WorthQuerySupportContributionPosture::DeclarationSupport => {
                WorthQuerySupportContributionAuthoring::declaration_support(
                    semantic_code,
                    self.detail,
                )
                .bind_to_installed_target(self.target)
            }
            WorthQuerySupportContributionPosture::DeclarationTraceability => {
                WorthQuerySupportContributionAuthoring::declaration_traceability(
                    semantic_code,
                    self.detail,
                )
                .bind_to_installed_target(self.target)
            }
            WorthQuerySupportContributionPosture::NarrowedSupport => unreachable!(),
        })?;

        materialize_common_lane(
            "support-traceability",
            WorthQueryDomainCapabilityTargetKind::IntentDeclaration,
            self.posture.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_intent_declaration_support_traceability_artifact,
        )?
    }

    pub fn materialize(
        self,
    ) -> Result<
        WorthQueryIntentDeclarationSupportTraceabilityArtifact,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_materialize().into_result()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentInvariantRegistrationDraft {
    pub(crate) target: WorthQueryInstalledDeclarationContributionTarget,
    pub(crate) semantic_code: String,
    pub(crate) invariant_catalog: InvariantCatalog,
}

impl WorthQueryIntentInvariantRegistrationDraft {
    pub fn because(
        self,
        detail: impl Into<String>,
    ) -> WorthQueryIntentInvariantRegistrationContribution {
        WorthQueryIntentInvariantRegistrationContribution {
            target: self.target,
            semantic_code: self.semantic_code,
            invariant_catalog: self.invariant_catalog,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentInvariantRegistrationContribution {
    target: WorthQueryInstalledDeclarationContributionTarget,
    semantic_code: String,
    invariant_catalog: InvariantCatalog,
    detail: String,
}

impl WorthQueryIntentInvariantRegistrationContribution {
    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryInvariantCatalogRegistrationArtifact>,
        IdentityDerivationDenial,
    > {
        let target = self.target.clone();
        let requested = WorthQueryInvariantCapabilityContributionAuthoring::invariant_registration(
            self.invariant_catalog,
            qualify_semantic_code(self.target.authority(), &self.semantic_code),
            self.detail,
        )?
        .bind_to_installed_target(self.target)?;

        Ok(materialize_common_lane(
            "invariant-capability",
            WorthQueryDomainCapabilityTargetKind::IntentDeclaration,
            WorthQueryInvariantCapabilityContributionPosture::InvariantRegistration.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_query_invariant_catalog_registration_artifact,
        )?)
    }

    pub fn materialize(
        self,
    ) -> Result<
        WorthQueryInvariantCatalogRegistrationArtifact,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_materialize()?.into_result()
    }
}
