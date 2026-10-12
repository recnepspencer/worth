use worth_query_execution::facade::integration::{
    WorthQueryPrimaryGraphIndexRefreshDenial, WorthQueryProductRelationalInstallation,
    WorthQueryRelationalSourceOwner,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::MergeExecutionOutcome;

use crate::effect_lifecycle::{
    execute_lowered_merge, MergeUniqueValueAuthority, RelationalEffectExecutionFailure,
};

use super::WorthQueryPrimaryGraphBackendHandle;

/// Physical ownership stages for a backend's one Relational runtime.
pub(super) enum WorthQueryBackendRelationalOwner {
    Unpublished(RelationalRuntime),
    ProductSource(WorthQueryRelationalSourceOwner),
    PrimaryGraph(WorthQueryPrimaryGraphBackendHandle),
}

impl WorthQueryBackendRelationalOwner {
    pub(super) fn with_runtime<T>(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        read: impl FnOnce(&RelationalRuntime) -> T,
    ) -> T {
        match self {
            Self::Unpublished(runtime) => read(runtime),
            Self::ProductSource(owner) => owner.with_runtime(request_execution, read),
            Self::PrimaryGraph(owner) => owner.with_runtime(request_execution, read),
        }
    }

    pub(super) fn execute_mutation<T, E>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        mutate: impl FnOnce(&mut RelationalRuntime) -> Result<T, E>,
    ) -> Result<Result<T, E>, WorthQueryPrimaryGraphIndexRefreshDenial> {
        match self {
            Self::Unpublished(runtime) => Ok(mutate(runtime)),
            Self::ProductSource(owner) => Ok(owner.with_runtime_mut(execution, mutate)),
            Self::PrimaryGraph(owner) => owner.execute_mutation(execution, mutate),
        }
    }

    /// Executes one lowered merge; only the primary graph carries an
    /// application schema whose unique fields the merge must keep.
    pub(super) fn execute_merge(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declaration: &crate::workflow::LoweredMergeWorkflowDeclaration,
    ) -> Result<
        Result<MergeExecutionOutcome, RelationalEffectExecutionFailure>,
        WorthQueryPrimaryGraphIndexRefreshDenial,
    > {
        let unschematized = |runtime: &mut RelationalRuntime| {
            execute_lowered_merge(
                execution,
                runtime,
                declaration,
                MergeUniqueValueAuthority::NoApplicationSchema,
            )
        };
        match self {
            Self::Unpublished(runtime) => Ok(unschematized(runtime)),
            Self::ProductSource(owner) => Ok(owner.with_runtime_mut(execution, unschematized)),
            Self::PrimaryGraph(owner) => owner.execute_merge(execution, declaration),
        }
    }

    pub(super) fn prepare_product_source(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<WorthQueryProductRelationalInstallation, super::WorthQueryProductSourceDenial> {
        let owner = match self {
            Self::Unpublished(_) => {
                return Err(super::WorthQueryProductSourceDenial::SourceNotInstalled)
            }
            Self::ProductSource(owner) => owner.clone(),
            Self::PrimaryGraph(owner) => {
                return owner
                    .prepare_product_source(request_execution)
                    .map_err(super::WorthQueryProductSourceDenial::Basis)
            }
        };
        let branch =
            owner.with_runtime(request_execution, |runtime| runtime.main_branch_identity());
        owner
            .prepare_product_source(request_execution, &branch)
            .map_err(super::WorthQueryProductSourceDenial::Basis)
    }
}

impl WorthQueryBackendRelationalOwner {
    pub(super) fn release_query_snapshot(
        &mut self,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) {
        match self {
            Self::Unpublished(runtime) => {
                runtime
                    .snapshots()
                    .release_snapshot(snapshot)
                    .expect("merge returns its exact snapshot to its owner");
            }
            Self::ProductSource(owner) => owner.release_query_snapshot(snapshot),
            Self::PrimaryGraph(owner) => owner.release_query_snapshot(snapshot),
        }
    }
}
