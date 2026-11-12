use crate::domain_capabilities::authoring::WorthQueryInvariantCapabilityContributionAuthoring;
use crate::domain_capabilities::canonical_runtime::materialize_graph_composition_domain_invariant_denial;
use crate::domain_capabilities::dx::checked::{
    WorthQueryCheckedDomainCapabilityOutcome, WorthQueryDomainCapabilityMaterializationError,
};
use crate::domain_capabilities::eligibility::{
    admit_eligible_domain_capability_contribution,
    evaluate_requested_domain_capability_contribution,
    prepare_admitted_domain_capability_contribution_for_materialization,
};
use crate::domain_capabilities::payloads::WorthQueryInvariantCapabilityContributionPosture;
use crate::domain_capabilities::{
    WorthQueryDomainCapabilityTargetKind, WorthQueryInstalledLowerRuntimeContributionTarget,
    WorthQueryRequestedInvariantCapabilityContribution,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::runtime::WorthQueryGraphCompositionDomainInvariantDenial;

use super::lower_runtime::WorthQueryLowerRuntimeDomainContributionSurface;
use super::materialization::materialize_common_lane;
use super::semantic_code::qualify_semantic_code;

impl WorthQueryLowerRuntimeDomainContributionSurface {
    #[allow(clippy::too_many_arguments)]
    pub fn denies_graph_invariant(
        self,
        invariant_family: impl Into<String>,
        declared_collections: impl IntoIterator<Item = impl Into<String>>,
        declared_symbols: impl IntoIterator<Item = impl Into<String>>,
        target_combination_families: impl IntoIterator<Item = impl Into<String>>,
        lifecycle_families: impl IntoIterator<Item = impl Into<String>>,
        program_digest: impl Into<String>,
        breadth_digest: impl Into<String>,
        counter_snapshot: impl Into<String>,
        semantic_code: impl AsRef<str>,
        detail: impl Into<String>,
    ) -> Result<WorthQueryLowerRuntimeGraphInvariantDenialContribution, IdentityDerivationDenial>
    {
        let target = self.target;
        let requested = WorthQueryInvariantCapabilityContributionAuthoring::graph_invariant_denial(
            invariant_family,
            declared_collections,
            declared_symbols,
            target_combination_families,
            lifecycle_families,
            program_digest,
            breadth_digest,
            counter_snapshot,
            qualify_semantic_code(target.authority(), semantic_code.as_ref()),
            detail,
        )?
        .bind_to_installed_target(target.clone())?;
        Ok(WorthQueryLowerRuntimeGraphInvariantDenialContribution { requested, target })
    }
}

pub struct WorthQueryLowerRuntimeGraphInvariantDenialContribution {
    requested: WorthQueryRequestedInvariantCapabilityContribution<
        WorthQueryInstalledLowerRuntimeContributionTarget,
    >,
    target: WorthQueryInstalledLowerRuntimeContributionTarget,
}

impl WorthQueryLowerRuntimeGraphInvariantDenialContribution {
    pub fn try_materialize(
        self,
    ) -> Result<
        WorthQueryCheckedDomainCapabilityOutcome<WorthQueryGraphCompositionDomainInvariantDenial>,
        crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
    > {
        let target = self.target;
        materialize_common_lane(
            "invariant-capability",
            WorthQueryDomainCapabilityTargetKind::LowerRuntimeBoundaryEnvelope,
            WorthQueryInvariantCapabilityContributionPosture::InvariantDenial.as_str(),
            self.requested,
            evaluate_requested_domain_capability_contribution,
            admit_eligible_domain_capability_contribution,
            |admitted| {
                prepare_admitted_domain_capability_contribution_for_materialization(
                    admitted, target,
                )
            },
            materialize_graph_composition_domain_invariant_denial,
        )
    }

    pub fn materialize(
        self,
    ) -> Result<
        WorthQueryGraphCompositionDomainInvariantDenial,
        WorthQueryDomainCapabilityMaterializationError,
    > {
        self.try_materialize()?.into_result()
    }
}
