use super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn release_conditional_runtime_resources(
        &mut self,
    ) {
        self.bridge
            .conditional_lifecycle()
            .close_conditional_resources();
        *self
            .conditional_operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.primary_provider.replace_conditional_commit_routes(
            std::iter::empty(),
            false,
            std::iter::empty(),
        );
    }
}

impl<Schema> Drop for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn drop(&mut self) {
        self.release_conditional_runtime_resources();
    }
}
