use crate::domain_capabilities::authoring::WorthQueryAftermathContributionAuthoring;
use crate::domain_capabilities::canonical_runtime::{
    materialize_admitted_projection_consumption, materialize_projection_consumption_contract,
    materialize_projection_consumption_eligibility, materialize_projection_consumption_review,
    materialize_projection_consumption_support_report,
    WorthQueryAftermathProjectionConsumptionReview,
};
use crate::domain_capabilities::dx::checked::{
    WorthQueryCheckedDomainCapabilityOutcome, WorthQueryDomainCapabilityMaterializationError,
};
use crate::domain_capabilities::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use crate::domain_capabilities::payloads::WorthQueryAftermathContributionPosture;
use crate::domain_capabilities::WorthQueryDomainCapabilityTargetKind;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::{
    AdmittedProjectionConsumption, MaterializedProjectionContract, ProjectMaterializedFacts,
    ProjectionConsumptionBindingContext, ProjectionConsumptionEligibility,
    ProjectionConsumptionSource, ProjectionConsumptionSupportReport,
};

use super::admitted_plan::WorthQueryAdmittedPlanDomainContributionSurface;
use super::materialization::materialize_common_lane;
use super::projection_contract_request::WorthQueryProjectionContractRequest;
use super::semantic_code::qualify_semantic_code;

impl WorthQueryAdmittedPlanDomainContributionSurface {
    pub fn establishes_projection_contract(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryProjectionContractRequest,
    ) -> WorthQueryAdmittedPlanAftermathDraft {
        let (source, binding, requested_facts) = request.into_parts();
        WorthQueryAdmittedPlanAftermathDraft::new(
            self.target,
            WorthQueryAftermathContributionPosture::EstablishesFact,
            semantic_code,
            source,
            binding,
            requested_facts,
        )
    }

    pub fn consumes_projection_contract(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryProjectionContractRequest,
    ) -> WorthQueryAdmittedPlanAftermathDraft {
        let (source, binding, requested_facts) = request.into_parts();
        WorthQueryAdmittedPlanAftermathDraft::new(
            self.target,
            WorthQueryAftermathContributionPosture::ConsumesFact,
            semantic_code,
            source,
            binding,
            requested_facts,
        )
    }

    pub fn declares_projection_residue(
        self,
        semantic_code: impl Into<String>,
        request: WorthQueryProjectionContractRequest,
    ) -> WorthQueryAdmittedPlanAftermathDraft {
        let (source, binding, requested_facts) = request.into_parts();
        WorthQueryAdmittedPlanAftermathDraft::new(
            self.target,
            WorthQueryAftermathContributionPosture::DeclaresResidue,
            semantic_code,
            source,
            binding,
            requested_facts,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanAftermathDraft {
    target: crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget,
    posture: WorthQueryAftermathContributionPosture,
    semantic_code: String,
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested_facts: ProjectMaterializedFacts,
}

impl WorthQueryAdmittedPlanAftermathDraft {
    fn new(
        target: crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget,
        posture: WorthQueryAftermathContributionPosture,
        semantic_code: impl Into<String>,
        source: ProjectionConsumptionSource,
        binding: ProjectionConsumptionBindingContext,
        requested_facts: ProjectMaterializedFacts,
    ) -> Self {
        Self {
            target,
            posture,
            semantic_code: semantic_code.into(),
            source,
            binding,
            requested_facts,
        }
    }

    pub fn because(self, detail: impl Into<String>) -> WorthQueryAdmittedPlanAftermathContribution {
        WorthQueryAdmittedPlanAftermathContribution {
            target: self.target,
            posture: self.posture,
            semantic_code: self.semantic_code,
            source: self.source,
            binding: self.binding,
            requested_facts: self.requested_facts,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedPlanAftermathContribution {
    target: crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget,
    posture: WorthQueryAftermathContributionPosture,
    semantic_code: String,
    source: ProjectionConsumptionSource,
    binding: ProjectionConsumptionBindingContext,
    requested_facts: ProjectMaterializedFacts,
    detail: String,
}

impl WorthQueryAdmittedPlanAftermathContribution {
    pub fn try_review(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryAftermathProjectionConsumptionReview>,
        IdentityDerivationDenial,
    > {
        Ok(self.materialize_checked(materialize_projection_consumption_review)?)
    }

    pub fn review(
        self,
    ) -> Result<
        WorthQueryAftermathProjectionConsumptionReview,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_review().into_result()
    }

    pub fn try_materialize_support_report(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<ProjectionConsumptionSupportReport>,
        IdentityDerivationDenial,
    > {
        Ok(self.materialize_checked(materialize_projection_consumption_support_report)?)
    }

    pub fn materialize_support_report(
        self,
    ) -> Result<ProjectionConsumptionSupportReport, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_materialize_support_report().into_result()
    }

    pub fn try_materialize_eligibility(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<ProjectionConsumptionEligibility>,
        IdentityDerivationDenial,
    > {
        Ok(self.materialize_checked(materialize_projection_consumption_eligibility)?)
    }

    pub fn materialize_eligibility(
        self,
    ) -> Result<ProjectionConsumptionEligibility, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_materialize_eligibility().into_result()
    }

    pub fn try_materialize_admitted(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<AdmittedProjectionConsumption>,
        IdentityDerivationDenial,
    > {
        Ok(self.materialize_checked(materialize_admitted_projection_consumption)?)
    }

    pub fn materialize_admitted(
        self,
    ) -> Result<AdmittedProjectionConsumption, WorthQueryDomainCapabilityMaterializationError> {
        self.try_materialize_admitted().into_result()
    }

    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<MaterializedProjectionContract>,
        IdentityDerivationDenial,
    > {
        Ok(self.materialize_checked(materialize_projection_consumption_contract)?)
    }

    pub fn materialize(
        self,
    ) -> Result<MaterializedProjectionContract, WorthQueryDomainCapabilityMaterializationError>
    {
        self.try_materialize()?.into_result()
    }

    fn into_requested(
        self,
    ) -> Result<
        crate::domain_capabilities::WorthQueryRequestedAftermathContribution<
            crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget,
        >,
        IdentityDerivationDenial,
    > {
        let semantic_code = qualify_semantic_code(self.target.authority(), &self.semantic_code);
        Ok(match self.posture {
            WorthQueryAftermathContributionPosture::EstablishesFact => {
                WorthQueryAftermathContributionAuthoring::establishes_projection_contract(
                    semantic_code,
                    self.detail,
                    self.source,
                    self.binding,
                    self.requested_facts,
                )?
                .bind_to_installed_target(self.target)?
            }
            WorthQueryAftermathContributionPosture::ConsumesFact => {
                WorthQueryAftermathContributionAuthoring::consumes_projection_contract(
                    semantic_code,
                    self.detail,
                    self.source,
                    self.binding,
                    self.requested_facts,
                )?
                .bind_to_installed_target(self.target)?
            }
            WorthQueryAftermathContributionPosture::DeclaresResidue => {
                WorthQueryAftermathContributionAuthoring::declares_projection_residue(
                    semantic_code,
                    self.detail,
                    self.source,
                    self.binding,
                    self.requested_facts,
                )?
                .bind_to_installed_target(self.target)?
            }
        })
    }

    fn materialize_checked<Success>(
        self,
        materialize: impl FnOnce(
            crate::domain_capabilities::WorthQueryMaterializationReadyAftermathContribution<
                crate::domain_capabilities::WorthQueryInstalledAdmittedPlanContributionTarget,
            >,
        ) -> crate::domain_capabilities::WorthQueryDomainCapabilityTransitionOutcome<Success>,
    ) -> Result<WorthQueryCheckedDomainCapabilityOutcome<Success>, IdentityDerivationDenial> {
        let posture = self.posture;
        let target = self.target.clone();
        let requested = self.into_requested()?;

        Ok(materialize_common_lane(
            "consequence-aftermath",
            WorthQueryDomainCapabilityTargetKind::AdmittedIntentPlan,
            posture.as_str(),
            requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            |ready| Ok(materialize(ready)),
        )?)
    }
}
