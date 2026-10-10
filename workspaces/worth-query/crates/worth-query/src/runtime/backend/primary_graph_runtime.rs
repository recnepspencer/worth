use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIndexRefreshDenial;
use worth_query_execution::facade::integration::WorthQueryPrimaryGraphIntegrationHandle;
use worth_relational::facade::runtime::RelationalRuntime;

/// Move-only ownership transfer for the Relational runtime that will become
/// Query's primary logical graph.
///
/// The private field prevents custom backends from manufacturing support for
/// primary-graph composition. Only Query's bridge-backed backend may surrender
/// the unpublished runtime it already owns.
#[doc(hidden)]
pub struct WorthQueryUnpublishedPrimaryGraphRuntime {
    runtime: RelationalRuntime,
}

impl WorthQueryUnpublishedPrimaryGraphRuntime {
    pub(in crate::runtime) fn new(runtime: RelationalRuntime) -> Self {
        Self { runtime }
    }

    pub(in crate::runtime) fn into_runtime(self) -> RelationalRuntime {
        self.runtime
    }
}

/// Opaque backend capability for the one execution-owned primary graph.
///
/// Product consumers receive neither this handle nor raw Relational access.
/// It exists only to reconnect Query's ordinary backend after execution has
/// consumed installation authority and published the graph.
#[doc(hidden)]
#[derive(Clone)]
pub struct WorthQueryPrimaryGraphBackendHandle {
    integration: WorthQueryPrimaryGraphIntegrationHandle,
}

impl WorthQueryPrimaryGraphBackendHandle {
    pub(super) fn prepare_product_source(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        worth_query_execution::facade::integration::WorthQueryProductRelationalInstallation,
        worth_relational::facade::branch::RelationalBranchBasisDenial,
    > {
        let branch = self
            .integration
            .with_runtime(request_execution, |runtime| runtime.main_branch_identity());
        self.integration
            .prepare_product_source(request_execution, &branch)
    }
    pub(in crate::runtime) fn new(integration: WorthQueryPrimaryGraphIntegrationHandle) -> Self {
        Self { integration }
    }

    pub(in crate::runtime) fn with_runtime<T>(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        self.integration.with_runtime(request_execution, read)
    }

    pub(in crate::runtime) fn execute_mutation<T, E>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> Result<T, E>,
    ) -> Result<Result<T, E>, WorthQueryPrimaryGraphIndexRefreshDenial> {
        self.integration
            .execute_mutation_with_index_refresh(execution, mutate)
    }

    pub(in crate::runtime) fn execute_merge(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        Result<
            worth_relational::facade::transactions::MergeExecutionOutcome,
            crate::effect_lifecycle::RelationalEffectExecutionFailure,
        >,
        WorthQueryPrimaryGraphIndexRefreshDenial,
    > {
        let unique_values =
            crate::effect_lifecycle::MergeUniqueValueAuthority::PrimaryGraph(&self.integration);
        self.integration
            .execute_mutation_with_index_refresh(execution, |runtime| {
                crate::effect_lifecycle::execute_lowered_merge(
                    execution,
                    runtime,
                    declaration,
                    unique_values,
                )
            })
    }
}

impl WorthQueryPrimaryGraphBackendHandle {
    pub(super) fn release_query_snapshot(
        &self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        self.integration.release_query_snapshot(snapshot);
    }
}
