use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryMutationReceipt,
    WorthQueryWorkspaceError,
};
use crate::runtime::{WorthQueryLiveArtifactTarget, WorthQueryRuntimeSourceAdapter};
use crate::schema_view::QuerySchemaView;

pub(crate) struct RepresentativeSourceAdapter;

impl WorthQueryRuntimeSourceAdapter for RepresentativeSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        _request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, _name: &str) -> Result<(), WorthQueryWorkspaceError> {
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        Ok(Vec::new())
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        _receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        Vec::new()
    }
}
