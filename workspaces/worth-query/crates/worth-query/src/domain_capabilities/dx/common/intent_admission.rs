use crate::domain_capabilities::authoring::WorthQueryAdmissionContributionAuthoring;
use crate::domain_capabilities::canonical_runtime::{
    materialize_canonical_admission_artifact, WorthQueryCanonicalAdmissionArtifact,
};
use crate::domain_capabilities::dx::checked::{
    WorthQueryCheckedDomainCapabilityOutcome, WorthQueryDomainCapabilityMaterializationError,
};
use crate::domain_capabilities::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture;
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryInstalledDeclarationContributionTarget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

use super::intent::WorthQueryIntentDomainContributionSurface;
use super::shared::{materialize_common_lane, qualify_semantic_code};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionDraft {
    target: WorthQueryInstalledDeclarationContributionTarget,
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: String,
}

impl WorthQueryIntentDomainContributionSurface {
    pub fn advises(self, semantic_code: impl Into<String>) -> WorthQueryIntentAdmissionDraft {
        WorthQueryIntentAdmissionDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture::Advisory,
            semantic_code,
        )
    }

    pub fn violates_invariant(
        self,
        semantic_code: impl Into<String>,
    ) -> WorthQueryIntentAdmissionDraft {
        WorthQueryIntentAdmissionDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture::Violation,
            semantic_code,
        )
    }
}

impl WorthQueryIntentAdmissionDraft {
    pub(crate) fn new(
        target: WorthQueryInstalledDeclarationContributionTarget,
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
    ) -> Self {
        Self {
            target,
            posture,
            semantic_code: semantic_code.into(),
        }
    }

    pub fn because(self, detail: impl Into<String>) -> WorthQueryIntentAdmissionContribution {
        WorthQueryIntentAdmissionContribution {
            target: self.target,
            posture: self.posture,
            semantic_code: self.semantic_code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryIntentAdmissionContribution {
    target: WorthQueryInstalledDeclarationContributionTarget,
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: String,
    detail: String,
}

impl WorthQueryIntentAdmissionContribution {
    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<
            WorthQueryCanonicalAdmissionArtifact<WorthQueryInstalledDeclarationContributionTarget>,
        >,
        IdentityDerivationDenial,
    > {
        let semantic_code = qualify_semantic_code(self.target.authority(), &self.semantic_code);
        let target = self.target.clone();
        let requested = match self.posture {
            WorthQueryAdmissionContributionPosture::Advisory => {
                WorthQueryAdmissionContributionAuthoring::advisory(semantic_code, self.detail)?
                    .bind_to_installed_target(self.target)
            }
            WorthQueryAdmissionContributionPosture::Violation => {
                WorthQueryAdmissionContributionAuthoring::violation(semantic_code, self.detail)?
                    .bind_to_installed_target(self.target)
            }
            WorthQueryAdmissionContributionPosture::SupportOnly => unreachable!(),
        };

        Ok(materialize_common_lane(
            "admission",
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
            |ready| {
                worth_proof::TransitionOutcome::Success(materialize_canonical_admission_artifact(
                    ready,
                ))
            },
        ))
    }

    pub fn materialize(
        self,
    ) -> Result<
        WorthQueryCanonicalAdmissionArtifact<WorthQueryInstalledDeclarationContributionTarget>,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_materialize().into_result()
    }
}
