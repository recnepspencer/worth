use super::WorthQueryRuntimeBuilder;

impl WorthQueryRuntimeBuilder {
    pub fn relational_product_source(
        mut self,
        runtime: worth_relational::facade::runtime::RelationalRuntime,
        graph_role: impl Into<std::sync::Arc<str>>,
    ) -> Result<Self, worth_relational::facade::bridge::RelationalBridgeSourceConfigurationError>
    {
        let owner =
            worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
                runtime, graph_role,
            )?;
        self.backend_parts = self.backend_parts.relational_source_owner(owner);
        Ok(self)
    }
}
