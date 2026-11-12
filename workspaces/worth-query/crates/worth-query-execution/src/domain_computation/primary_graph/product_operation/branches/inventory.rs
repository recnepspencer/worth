//! Fresh selectors come from World's admitted cells, never stored live descriptors.
use super::WorthQueryApplicationProductBranches;
use crate::basis::WorthQueryProductBranch;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ObservedProductBranches, ProductBranchInventoryDenial, RuntimeWorldServiceDenial,
};
/// Retains the actual request-funded World observations for this membership cut.
/// Concurrently moving branches need ordinary selection before domain work.
pub struct WorthQueryProductBranchInventory {
    observed: ObservedProductBranches,
}
impl WorthQueryProductBranchInventory {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = WorthQueryProductBranch> + '_ {
        self.observed
            .branches()
            .iter()
            .map(|branch| WorthQueryProductBranch::from_occurrence(branch.lifecycle_incarnation()))
    }
}
#[derive(Debug)]
pub enum WorthQueryProductBranchInventoryDenial {
    ForeignPhase(WorthQueryForeignAdvancementPhase),
    World(RuntimeWorldServiceDenial<ProductBranchInventoryDenial>),
}
impl<Schema: ApplicationSchema> WorthQueryApplicationProductBranches<'_, Schema> {
    /// Discover current branch selectors, including siblings admitted from Home.
    /// Persisted comparison descriptions do not replace the Complete Home image.
    pub fn inventory(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryProductBranchInventory, WorthQueryProductBranchInventoryDenial> {
        let runtime = &self.application.runtime.product_runtime;
        let execution = phase
            .execution_request_for(runtime)
            .map_err(WorthQueryProductBranchInventoryDenial::ForeignPhase)?;
        let observed = runtime
            .owner
            .observation_port()
            .observe_installed_product_branches(&execution)
            .map_err(WorthQueryProductBranchInventoryDenial::World)?;
        Ok(WorthQueryProductBranchInventory { observed })
    }
}
