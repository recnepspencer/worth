//! Lowers an attempt's bound output names against its own commit basis.
use worth_query_execution_lineage::facade::persistent_naming::{
    BoundOperationNaming, WorthQueryOutputNameDenial,
};
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductPublicationBindingInternalPort as _;
use worth_relational::facade::lineage::CommitNamingPlan;

impl super::WorthQueryPrimaryGraphProvider {
    /// Lowers `naming` against the derived name index at `product`'s exact
    /// Relational observation, the basis the attempt's transaction opens on.
    ///
    /// Complexity: see `BoundOperationNaming::lower`: O(E log E + N) for E
    /// named outputs and N held names, never O(names) or O(commits).
    pub(crate) fn lower_operation_names(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        naming: &BoundOperationNaming,
    ) -> Result<CommitNamingPlan, WorthQueryOutputNameDenial> {
        let layout = self.graph.layout_ref();
        let names = layout.persistent_names().ok_or_else(|| {
            WorthQueryOutputNameDenial::NameIndexUnavailable {
                detail: "the installed schema declares no persistent-name schemes".to_owned(),
            }
        })?;
        let observation = product
            .observation()
            .basis()
            .relational_basis()
            .observation();
        self.graph.with_runtime(execution, |runtime| {
            runtime
                .with_persistent_names_at_observation(execution, &observation, |view| {
                    naming.lower(names.catalog(), view)
                })
                .map_err(|denial| WorthQueryOutputNameDenial::NameIndexUnavailable {
                    detail: format!("{denial:?}"),
                })?
        })
    }
}
