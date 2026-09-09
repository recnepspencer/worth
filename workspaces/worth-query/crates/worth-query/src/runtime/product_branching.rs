use super::WorthQueryRuntime;

impl WorthQueryRuntime {
    pub(crate) fn create_product_branch(
        &self,
        source: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
        intent: worth_query_execution::facade::runtime::ProductBranchCreationIntent,
        cancellation: &worth_query_execution::facade::runtime::RuntimeWorldCancellationToken,
    ) -> Result<
        worth_query_execution::facade::runtime::RuntimeWorldBranchCreationOutcome,
        WorthQueryProductBranchCreationDenial,
    > {
        let product = self
            .installed_product
            .as_ref()
            .ok_or(WorthQueryProductBranchCreationDenial::RuntimeUnavailable)?;
        product
            .validate_selected_source(source, None)
            .map_err(|_| WorthQueryProductBranchCreationDenial::ProductBasisRequired)?;
        product
            .world
            .create_product_branch(source, intent, cancellation)
            .map_err(WorthQueryProductBranchCreationDenial::Creation)
    }
}

#[derive(Debug)]
pub enum WorthQueryProductBranchCreationDenial {
    RuntimeUnavailable,
    ProductBasisRequired,
    Creation(worth_query_execution::facade::runtime::WorthQueryProductBranchCreationDenial),
}
