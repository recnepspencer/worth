//! Retire the invariant authority's original graph source through that owner.
use super::WorthQueryApplicationInvariantProjectionAuthority;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryOperationHomeRetirementProgress,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::runtime::RelationalRuntime;

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<WorthQueryOperationHomeRetirementProgress, WorthQueryHomeRetirementDenial> {
        self.graph.advance_home_retirement(request, native)
    }
}
