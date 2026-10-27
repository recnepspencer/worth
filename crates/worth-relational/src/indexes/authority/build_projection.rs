use super::{IndexProjectionSource, RelationalRuntime};

pub(super) enum IndexBuildProjection {
    Exact(crate::runtime::VisibilityProjectionView),
    Historical(crate::runtime::VisibilityProjectionView),
}

impl IndexBuildProjection {
    /// Select immutable source metadata before request-paid index record reads.
    pub(super) fn select(
        runtime: &RelationalRuntime,
        branch_id: &crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Self>, crate::branch::RelationalBranchBasisDenial> {
        if version_id == runtime.current_version_id() {
            return runtime
                .read_truth()
                .project_branch_head(branch_id, version_id)
                .map(|projection| projection.map(Self::Exact));
        }
        runtime
            .read_truth()
            .try_project_historical_version(version_id)
            .map(|projection| Some(Self::Historical(projection)))
    }

    pub(super) fn demand(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Result<(), crate::execution::RelationalExecutionDenialCause> {
        let view = match self {
            Self::Exact(view) | Self::Historical(view) => view,
        };
        if let Some(root) = view.selected_root() {
            root.demand_all_full(execution, &runtime.services.symbols)?;
        }
        view.refresh_symbol_meanings(&runtime.services.symbols);
        Ok(())
    }

    pub(super) fn source(&self) -> Option<IndexProjectionSource<'_>> {
        match self {
            Self::Exact(projection) => IndexProjectionSource::exact(projection),
            Self::Historical(projection) => IndexProjectionSource::historical(projection),
        }
    }
}
