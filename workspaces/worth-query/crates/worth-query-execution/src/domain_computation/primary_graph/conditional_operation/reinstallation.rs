use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledPackageIndex, WorthQueryInstalledPackageIndexRelation,
};

use super::signal_decision_reentry::WorthQueryConditionalTruthBasis;
use super::{
    WorthQueryConditionalRuntimeInstallationDenial,
    WorthQueryConditionalRuntimeInstallationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

#[derive(Clone)]
pub struct WorthQueryConditionalRuntimeReinstallationReceipt {
    lower_runtime_reconstitution:
        worth_runtime_bridge::facade::BridgeConditionalRuntimeReconstitutionReport,
    reconstructed_binding_count: usize,
    reconstructed_intent_count: usize,
    examined_candidate_count: usize,
    projected_record_count: usize,
    projected_field_count: usize,
    total_work_units: usize,
    successor_invalidation_installation: super::super::WorthQueryGranularInvalidationInstallation,
}

impl std::fmt::Debug for WorthQueryConditionalRuntimeReinstallationReceipt {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryConditionalRuntimeReinstallationReceipt")
            .field(
                "lower_runtime_reconstitution",
                &self.lower_runtime_reconstitution,
            )
            .field(
                "reconstructed_binding_count",
                &self.reconstructed_binding_count,
            )
            .field(
                "reconstructed_intent_count",
                &self.reconstructed_intent_count,
            )
            .field("total_work_units", &self.total_work_units)
            .finish_non_exhaustive()
    }
}

impl WorthQueryConditionalRuntimeReinstallationReceipt {
    pub const fn lower_runtime_reconstitution(
        &self,
    ) -> worth_runtime_bridge::facade::BridgeConditionalRuntimeReconstitutionReport {
        self.lower_runtime_reconstitution
    }

    pub const fn reconstructed_binding_count(&self) -> usize {
        self.reconstructed_binding_count
    }

    /// Authoritative temporal intents projected during this reconstructive pass.
    pub const fn reconstructed_intent_count(&self) -> usize {
        self.reconstructed_intent_count
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.examined_candidate_count
    }
    pub const fn projected_record_count(&self) -> usize {
        self.projected_record_count
    }
    pub const fn projected_field_count(&self) -> usize {
        self.projected_field_count
    }
    pub const fn total_work_units(&self) -> usize {
        self.total_work_units
    }

    /// Exact successor installation minted by this reconstructive transition.
    #[doc(hidden)]
    pub const fn successor_invalidation_installation(
        &self,
    ) -> &super::super::WorthQueryGranularInvalidationInstallation {
        &self.successor_invalidation_installation
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Rebuilds volatile Bridge/Signal state only when the presented
    /// installation is exactly the installation already owning this runtime.
    pub fn reinstall_conditional_runtime(
        &mut self,
        product_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryConditionalRuntimeReinstallationReceipt,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        let current = self.runtime.retain_installed_packages();
        self.reinstall_conditional_runtime_for_installation(current, product_branch)
    }

    /// Rebuilds derived conditional state for one explicitly presented
    /// installation candidate. A changed generation or meaning cannot inherit
    /// incumbent typed bindings and therefore fails closed with RebindRequired.
    pub fn reinstall_conditional_runtime_for_installation(
        &mut self,
        candidate: Arc<WorthQueryInstalledPackageIndex>,
        product_branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<
        WorthQueryConditionalRuntimeReinstallationReceipt,
        WorthQueryConditionalRuntimeInstallationDenial,
    > {
        require_equivalent_installation(self.runtime.installed_packages(), &candidate)?;
        let selected = self
            .on_branch(product_branch)
            .select()
            .map_err(|denial| bridge_denial(format!("product selection failed: {denial:?}")))?;
        let truth = WorthQueryConditionalTruthBasis::from_selected(selected);
        let mut bridge_candidate = self
            .bridge
            .conditional()
            .prepare_conditional_reconstitution(truth.signal_basis())
            .map_err(|denial| bridge_denial(denial.detail()))?;
        let mut registry = self
            .conditional_operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        let mut prepared = registry.prepare_derived_runtime_reinstallation(
            self,
            &mut bridge_candidate,
            truth.product(),
        )?;
        let reconstructed_intent_count = prepared
            .values()
            .map(|binding| binding.reconstructed_intent_count)
            .sum();
        registry.reconcile_prepared_runtime_reinstallation(&mut bridge_candidate, &mut prepared)?;
        let lower_runtime_reconstitution = self
            .bridge
            .conditional_lifecycle()
            .activate_conditional_reconstitution(bridge_candidate)
            .map_err(|denial| bridge_denial(denial.detail()))?;
        registry.apply_derived_runtime_reinstallation(prepared, self);
        self.granular_invalidation.advance_runtime_generation();
        let work = registry.reconstruction_work();
        Ok(WorthQueryConditionalRuntimeReinstallationReceipt {
            lower_runtime_reconstitution,
            reconstructed_binding_count: registry.len(),
            reconstructed_intent_count,
            examined_candidate_count: work.examined_candidates,
            projected_record_count: work.projected_records,
            projected_field_count: work.projected_fields,
            total_work_units: work.total_work_units,
            successor_invalidation_installation: self.granular_invalidation.current(),
        })
    }
}

fn require_equivalent_installation(
    current: &WorthQueryInstalledPackageIndex,
    candidate: &WorthQueryInstalledPackageIndex,
) -> Result<(), WorthQueryConditionalRuntimeInstallationDenial> {
    match current.relation_to(candidate) {
        WorthQueryInstalledPackageIndexRelation::EquivalentGeneration => Ok(()),
        WorthQueryInstalledPackageIndexRelation::ExactSuccessor => Err(rebind(
            "successor installation requires fresh typed conditional bindings",
        )),
        WorthQueryInstalledPackageIndexRelation::SameGenerationMeaningChanged => Err(rebind(
            "candidate installation changed meaning within the current generation",
        )),
        WorthQueryInstalledPackageIndexRelation::ForeignRuntime => {
            Err(rebind("candidate installation belongs to another runtime"))
        }
        WorthQueryInstalledPackageIndexRelation::NonSuccessorGeneration => Err(rebind(
            "candidate installation is not the exact current or successor generation",
        )),
    }
}

fn rebind(detail: impl Into<String>) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::RebindRequired,
        detail,
    )
}

fn bridge_denial(detail: impl Into<String>) -> WorthQueryConditionalRuntimeInstallationDenial {
    WorthQueryConditionalRuntimeInstallationDenial::new(
        WorthQueryConditionalRuntimeInstallationDenialKind::BridgeRejected,
        detail,
    )
}
