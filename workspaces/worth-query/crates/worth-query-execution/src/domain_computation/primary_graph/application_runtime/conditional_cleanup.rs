use super::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation::primary_graph) fn release_conditional_runtime_resources(
        &mut self,
    ) {
        self.runtime
            .bridge
            .conditional_lifecycle()
            .close_conditional_resources();
        *self
            .conditional_operations
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Default::default();
        self.installed_resources.clear();
    }
}

impl<Schema> Drop for WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    fn drop(&mut self) {
        // The local owner drops first even if cleanup unwinds. Move later
        // custody into its original field positions before cleanup can panic.
        let execution = self.runtime.execution.take();
        self.post_conditional_teardown.take_from(&mut self.runtime);
        self.workflow_teardown.take_from(&mut self.runtime);
        self.release_conditional_runtime_resources();
        drop(execution);
    }
}
