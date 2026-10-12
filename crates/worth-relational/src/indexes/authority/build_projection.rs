use super::{IndexProjectionSource, RelationalRuntime};

pub(super) enum IndexBuildProjection {
    Exact(crate::runtime::VisibilityProjectionView),
    Historical(crate::runtime::VisibilityProjectionView),
}

impl IndexBuildProjection {
    pub(super) fn select(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        branch_id: &crate::history::data::BranchId,
        version_id: crate::identity::data::VersionId,
    ) -> Result<Option<Self>, crate::branch::RelationalBranchBasisDenial> {
        if version_id == runtime.current_version_id() {
            return runtime
                .read_truth()
                .project_branch_head(execution, branch_id, version_id)
                .map(|projection| projection.map(Self::Exact));
        }
        runtime
            .read_truth()
            .try_project_historical_version(execution, version_id)
            .map(|projection| Some(Self::Historical(projection)))
    }

    pub(super) fn source(&self) -> Option<IndexProjectionSource<'_>> {
        match self {
            Self::Exact(projection) => IndexProjectionSource::exact(projection),
            Self::Historical(projection) => IndexProjectionSource::historical(projection),
        }
    }
}
