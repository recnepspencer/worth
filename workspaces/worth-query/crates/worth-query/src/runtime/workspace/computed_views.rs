//! Declare maintained or reusable computed views on this workspace.
use super::*;
impl WorthQueryWorkspace {
    pub fn computed<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        name: impl Into<String>,
        view: impl FnOnce(WorthQueryComputedBuilder) -> WorthQueryComputedBuilder,
        maintainer: impl WorthQueryDerivedViewMaintainer + 'static,
    ) -> Result<WorthQueryDerivedViewHandle<T>, WorthQueryRuntimeError> {
        let name = name.into();
        let view = view(WorthQueryComputedBuilder::surface(&name)).build()?;
        self.runtime
            .declare_maintained_derived_view(execution, view, maintainer)
    }

    pub fn computed_view<T>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: WorthQueryDerivedView,
        maintainer: impl WorthQueryDerivedViewMaintainer + 'static,
    ) -> Result<WorthQueryDerivedViewHandle<T>, WorthQueryRuntimeError> {
        self.runtime
            .declare_maintained_derived_view(execution, view, maintainer)
    }

    pub fn computed_definition(
        &mut self,
        name: impl Into<String>,
        view: impl FnOnce(WorthQueryComputedBuilder) -> WorthQueryComputedBuilder,
    ) -> Result<WorthQueryDerivedView, WorthQueryRuntimeError> {
        let name = name.into();
        let view = view(WorthQueryComputedBuilder::surface(&name)).build()?;
        self.runtime.declare_derived_view(view)
    }
}
