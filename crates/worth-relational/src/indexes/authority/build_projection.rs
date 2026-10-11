use super::{IndexProjectionSource, RelationalRuntime};

pub(super) enum IndexBuildProjection<'runtime> {
    Exact(crate::runtime::VisibilityProjectionView<'runtime>),
    Historical(crate::runtime::VisibilityProjectionView<'runtime>),
}

impl<'runtime> IndexBuildProjection<'runtime> {
    pub(super) fn select(
        runtime: &'runtime RelationalRuntime,
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

    pub(super) fn source(&self) -> Option<IndexProjectionSource<'_, 'runtime>> {
        match self {
            Self::Exact(projection) => IndexProjectionSource::exact(projection),
            Self::Historical(projection) => IndexProjectionSource::historical(projection),
        }
    }
}
