use super::WorthQueryRuntimeBuilder;

impl WorthQueryRuntimeBuilder {
    pub fn relational_product_source(
        mut self,
        runtime: worth_relational::facade::runtime::RelationalRuntime,
        graph_role: impl Into<std::sync::Arc<str>>,
    ) -> Result<
        Self,
        worth_query_execution::facade::integration::WorthQueryRelationalSourceInstallationDenial,
    > {
        let owner =
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
                runtime,
                graph_role,
                self.product_world_resources.invalidation_resources(),
            )?;
        self.backend_parts = self.backend_parts.relational_source_owner(owner);
        Ok(self)
    }
}
