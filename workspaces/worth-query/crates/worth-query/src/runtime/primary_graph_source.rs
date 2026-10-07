use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryMutationReceipt,
    WorthQueryWorkspaceError,
};
use crate::runtime::{WorthQueryLiveArtifactTarget, WorthQueryRuntimeSourceAdapter};
use crate::schema_view::QuerySchemaView;

/// Domain-owned projection from the authoritative primary graph into Query's
/// live source rows.
///
/// The projector owns schema meaning. The adapter owns the graph boundary and
/// passes the already-admitted semantic scope through unchanged. Implementors
/// must use source-owned exact lookup for granular reads; scanning a broader
/// target and filtering afterward is not a valid implementation.
pub trait WorthQueryPrimaryGraphSourceProjection: 'static {
    fn project_live_target(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError>;

    fn project_granular_scope(
        &self,
        graph: &worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
        target: &WorthQueryLiveArtifactTarget,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError>;
}

/// Query source adapter bound to the same primary graph that minted granular
/// invalidation delivery.
///
/// This adapter grants source access only. Runtime identity and invalidation
/// authority remain with the execution-owned installation retained separately
/// by `WorthQueryRuntime`.
pub struct WorthQueryPrimaryGraphSourceAdapter<P> {
    installation:
        worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    graph: worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle,
    projection: P,
}

impl<P> WorthQueryPrimaryGraphSourceAdapter<P> {
    pub fn new(
        installation: &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
        projection: P,
    ) -> Self {
        Self {
            installation: installation.clone(),
            graph: installation.retain_primary_graph_integration_handle(),
            projection,
        }
    }
}

impl<P> WorthQueryRuntimeSourceAdapter for WorthQueryPrimaryGraphSourceAdapter<P>
where
    P: WorthQueryPrimaryGraphSourceProjection,
{
    fn primary_graph_invalidation_installation(
        &self,
    ) -> Option<
        &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    > {
        Some(&self.installation)
    }

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
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError> {
        self.graph.with_runtime(|_| ())?;
        self.projection.project_live_target(&self.graph, target)
    }

    fn live_entities_for_granular_scope(
        &self,
        target: &WorthQueryLiveArtifactTarget,
        scope: &crate::live::WorthQueryMaintenanceScope,
        basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        self.graph.with_runtime(|_| ())?;
        let rows = self
            .projection
            .project_granular_scope(&self.graph, target, scope, basis)?;
        Ok(rows)
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
