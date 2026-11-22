use worth_query_execution_lineage::facade::ports::PrimaryGraphInternalPort as _;
use worth_query_execution_runtime::facade::execution_runtime as rt;
use worth_query_execution_runtime::facade::ports::ExecutionRuntimeInternalPort as _;

pub mod product_world;
pub use rt::source_invalidation;
pub use rt::{
    WorthQueryApplicationQueryResourceProfile, WorthQueryBranchCoordinationResourceProfile,
    WorthQueryExecutionInstallationAuthority, WorthQueryExecutionInstallationCommitDenial,
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstallation,
    WorthQueryExecutionRuntimeInstaller, WorthQueryInvalidationResources,
    WorthQueryOutputDemandLimits, WorthQueryOutputDemandResourceProfile,
    WorthQueryRuntimeAuthorityIdentity,
};

impl crate::ports::ExecutionRuntimePrimaryGraphPort for WorthQueryExecutionRuntime {
    fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>
    {
        <Self as QueryExecutionRuntimeInternalPort>::retain_primary_graph_integration_handle(self)
    }
}

/// Crate-boundary port for `WorthQueryExecutionRuntime` primary-graph downcast
/// consumed by `worth-query-execution-query` and `worth-query-execution`.
/// Never exported on `worth-query-execution::facade` or Host.
pub trait QueryExecutionRuntimeInternalPort: Sized {
    fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph>;
    fn primary_graph_mut(
        &mut self,
    ) -> Option<&mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraph>;
    fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>;
    fn install_primary_graph(
        &mut self,
        graph: crate::domain_computation::primary_graph::WorthQueryPrimaryGraph,
    );
}

impl QueryExecutionRuntimeInternalPort for WorthQueryExecutionRuntime {
    fn primary_graph(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        self.primary_graph_any().and_then(|any| any.downcast_ref())
    }

    fn primary_graph_mut(
        &mut self,
    ) -> Option<&mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraph> {
        self.primary_graph_any_mut()
            .and_then(|any| any.downcast_mut())
    }

    fn retain_primary_graph_integration_handle(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle>
    {
        self.primary_graph().map(
            crate::domain_computation::primary_graph::WorthQueryPrimaryGraph::integration_handle,
        )
    }

    fn install_primary_graph(
        &mut self,
        graph: crate::domain_computation::primary_graph::WorthQueryPrimaryGraph,
    ) {
        self.install_primary_graph_any(Box::new(graph));
    }
}

#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
