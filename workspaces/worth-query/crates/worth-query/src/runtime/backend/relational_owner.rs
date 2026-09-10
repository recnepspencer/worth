use worth_query_execution::facade::integration::{
    WorthQueryPrimaryGraphIndexRefreshDenial, WorthQueryProductRelationalInstallation,
    WorthQueryRelationalSourceOwner,
};
use worth_relational::facade::runtime::RelationalRuntime;

use super::WorthQueryPrimaryGraphBackendHandle;

/// Physical ownership stages for a backend's one Relational runtime.
pub(super) enum WorthQueryBackendRelationalOwner {
    Unpublished(RelationalRuntime),
    ProductSource(WorthQueryRelationalSourceOwner),
    PrimaryGraph(WorthQueryPrimaryGraphBackendHandle),
}

impl WorthQueryBackendRelationalOwner {
    pub(super) fn with_runtime<T>(&self, read: impl FnOnce(&RelationalRuntime) -> T) -> T {
        match self {
            Self::Unpublished(runtime) => read(runtime),
            Self::ProductSource(owner) => owner.with_runtime(read),
            Self::PrimaryGraph(owner) => owner.with_runtime(read),
        }
    }

    pub(super) fn execute_mutation<T, E>(
        &mut self,
        mutate: impl FnOnce(&mut RelationalRuntime) -> Result<T, E>,
    ) -> Result<Result<T, E>, WorthQueryPrimaryGraphIndexRefreshDenial> {
        match self {
            Self::Unpublished(runtime) => Ok(mutate(runtime)),
            Self::ProductSource(owner) => Ok(owner.with_runtime_mut(mutate)),
            Self::PrimaryGraph(owner) => owner.execute_mutation(mutate),
        }
    }

    pub(super) fn prepare_product_source(
        &self,
    ) -> Result<WorthQueryProductRelationalInstallation, super::WorthQueryProductSourceDenial> {
        let owner = match self {
            Self::Unpublished(_) => {
                return Err(super::WorthQueryProductSourceDenial::SourceNotInstalled)
            }
            Self::ProductSource(owner) => owner.clone(),
            Self::PrimaryGraph(owner) => {
                return owner
                    .prepare_product_source()
                    .map_err(super::WorthQueryProductSourceDenial::Basis)
            }
        };
        let branch = owner.with_runtime(|runtime| runtime.main_branch_identity());
        owner
            .prepare_product_source(&branch)
            .map_err(super::WorthQueryProductSourceDenial::Basis)
    }
}
