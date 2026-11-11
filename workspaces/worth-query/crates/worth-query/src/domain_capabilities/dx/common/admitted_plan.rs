use crate::domain_capabilities::authoring::{
    WorthQueryAdmissionContributionAuthoring, WorthQueryContinuityContributionAuthoring,
};
use crate::domain_capabilities::canonical_runtime::{
    materialize_runtime_admission_decision, materialize_runtime_continuity_evidence,
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
    WorthQueryAdmissionContributionPosture, WorthQueryContinuityContributionPosture,
};
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryInstalledAdmittedPlanContributionTarget,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::WorthQueryIntentAdmissionDecision;
use crate::runtime::WorthQueryContinuityMutationEvidence;

use super::materialization::materialize_common_lane;
use super::semantic_code::qualify_semantic_code;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanDomainContributionSurface {
    pub(crate) target: WorthQueryInstalledAdmittedPlanContributionTarget,
}

impl WorthQueryAdmittedPlanDomainContributionSurface {
    pub fn advises(self, semantic_code: impl Into<String>) -> WorthQueryAdmittedPlanAdmissionDraft {
        WorthQueryAdmittedPlanAdmissionDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture::Advisory,
            semantic_code,
        )
    }

    pub fn violates(
        self,
        semantic_code: impl Into<String>,
    ) -> WorthQueryAdmittedPlanAdmissionDraft {
        WorthQueryAdmittedPlanAdmissionDraft::new(
            self.target,
            crate::domain_capabilities::payloads::WorthQueryAdmissionContributionPosture::Violation,
            semantic_code,
        )
    }

    pub fn preserves_continuity(
        self,
        semantic_code: impl Into<String>,
        prior_authoritative_identity: impl Into<String>,
        successor_authoritative_identity: impl Into<String>,
    ) -> WorthQueryAdmittedPlanContinuityDraft {
        WorthQueryAdmittedPlanContinuityDraft {
            target: self.target,
            semantic_code: semantic_code.into(),
            prior_authoritative_identity: prior_authoritative_identity.into(),
            successor_authoritative_identity: successor_authoritative_identity.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanAdmissionDraft {
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: String,
}

impl WorthQueryAdmittedPlanAdmissionDraft {
    pub(crate) fn new(
        target: WorthQueryInstalledAdmittedPlanContributionTarget,
        posture: WorthQueryAdmissionContributionPosture,
        semantic_code: impl Into<String>,
    ) -> Self {
        Self {
            target,
            posture,
            semantic_code: semantic_code.into(),
        }
    }

    pub fn because(self, detail: impl Into<String>) -> WorthQueryAdmittedPlanAdmissionContribution {
        WorthQueryAdmittedPlanAdmissionContribution {
            target: self.target,
            posture: self.posture,
            semantic_code: self.semantic_code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanAdmissionContribution {
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
    posture: WorthQueryAdmissionContributionPosture,
    semantic_code: String,
    detail: String,
}

impl WorthQueryAdmittedPlanAdmissionContribution {
    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryIntentAdmissionDecision>,
        IdentityDerivationDenial,
    > {
        let target = self.target.clone();
        let requested = (match self.posture {
            WorthQueryAdmissionContributionPosture::Advisory => {
                WorthQueryAdmissionContributionAuthoring::advisory(
                    qualify_semantic_code(self.target.authority(), &self.semantic_code),
                    self.detail,
                )?
                .bind_to_installed_target(self.target)
            }
            WorthQueryAdmissionContributionPosture::Violation => {
                WorthQueryAdmissionContributionAuthoring::violation(
                    qualify_semantic_code(self.target.authority(), &self.semantic_code),
                    self.detail,
                )?
                .bind_to_installed_target(self.target)
            }
            WorthQueryAdmissionContributionPosture::SupportOnly => unreachable!(),
        })?;

        materialize_common_lane(
            "admission",
            WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
            self.posture.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            |ready| Ok(materialize_runtime_admission_decision(ready)),
        )
    }

    pub fn materialize(
        self,
    ) -> Result<WorthQueryIntentAdmissionDecision, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_materialize()?.into_result()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanContinuityDraft {
    pub(crate) target: WorthQueryInstalledAdmittedPlanContributionTarget,
    pub(crate) semantic_code: String,
    pub(crate) prior_authoritative_identity: String,
    pub(crate) successor_authoritative_identity: String,
}

impl WorthQueryAdmittedPlanContinuityDraft {
    pub fn because(
        self,
        detail: impl Into<String>,
    ) -> WorthQueryAdmittedPlanContinuityContribution {
        WorthQueryAdmittedPlanContinuityContribution {
            target: self.target,
            semantic_code: self.semantic_code,
            prior_authoritative_identity: self.prior_authoritative_identity,
            successor_authoritative_identity: self.successor_authoritative_identity,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanContinuityContribution {
    target: WorthQueryInstalledAdmittedPlanContributionTarget,
    semantic_code: String,
    prior_authoritative_identity: String,
    successor_authoritative_identity: String,
    detail: String,
}

impl WorthQueryAdmittedPlanContinuityContribution {
    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryContinuityMutationEvidence>,
        IdentityDerivationDenial,
    > {
        let target = self.target.clone();
        let requested = WorthQueryContinuityContributionAuthoring::preserved_rebind(
            self.prior_authoritative_identity,
            self.successor_authoritative_identity,
            qualify_semantic_code(self.target.authority(), &self.semantic_code),
            self.detail,
        )?
        .bind_to_installed_target(self.target)?;

        materialize_common_lane(
            "continuity-lineage",
            WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
            WorthQueryContinuityContributionPosture::Preserved.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_runtime_continuity_evidence,
        )
    }

    pub fn materialize(
        self,
    ) -> Result<WorthQueryContinuityMutationEvidence, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_materialize()?.into_result()
    }
}
