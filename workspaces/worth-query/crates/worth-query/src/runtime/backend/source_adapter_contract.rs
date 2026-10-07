use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::memory_workspace::{
    WorthQueryEntity, WorthQueryLivePatch, WorthQueryLiveViewHandle, WorthQueryMutationReceipt,
    WorthQueryWorkspaceError,
};
use crate::runtime::WorthQueryLiveArtifactTarget;
use crate::schema_view::QuerySchemaView;

/// Backend-owned source access for live Query materializations.
pub trait WorthQueryRuntimeSourceAdapter {
    /// Returns the exact primary-runtime installation retained by this source
    /// adapter, when it is a primary-graph source.
    ///
    /// This is composition identity only. It grants no invalidation or read
    /// authority and exists so runtime construction can reject A/B graph
    /// substitution before attaching either source.
    #[doc(hidden)]
    fn primary_graph_invalidation_installation(
        &self,
    ) -> Option<
        &worth_query_execution::facade::primary_graph::WorthQueryGranularInvalidationInstallation,
    > {
        None
    }

    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError>;

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError>;

    fn live_entities_for_target(
        &self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Result<Vec<WorthQueryEntity>, crate::memory_workspace::WorthQueryWorkspaceError>;

    /// Reads only the admitted semantic granule from the source of truth.
    ///
    /// Implementations must not satisfy this operation by scanning the full
    /// target and filtering afterward. Backends without an exact source-owned
    /// lookup leave the default typed denial in place.
    fn live_entities_for_granular_scope(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
        _scope: &crate::live::WorthQueryMaintenanceScope,
        _basis: &crate::runtime::WorthQueryGranularSourceReadBasis,
    ) -> Result<Vec<WorthQueryEntity>, WorthQueryWorkspaceError> {
        Err(WorthQueryWorkspaceError::new(
            "this source adapter has no exact granular live-source reader",
        ))
    }

    fn drain_live_patches_for_target(
        &mut self,
        target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch>;

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget>;
}
