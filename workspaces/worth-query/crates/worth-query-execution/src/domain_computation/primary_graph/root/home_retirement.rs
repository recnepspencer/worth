//! Delegate retirement legality to the actual source that owns Native custody.
use super::WorthQueryPrimaryGraph;
use crate::domain_computation::primary_graph::WorthQueryHomeRetirementDenial;

impl WorthQueryPrimaryGraph {
    pub(in crate::domain_computation::primary_graph) fn advance_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        super::super::WorthQueryOperationHomeRetirementProgress,
        WorthQueryHomeRetirementDenial,
    > {
        self.source_owner.advance_home_retirement(request)
    }

    pub(in crate::domain_computation::primary_graph) fn preflight_home_retirement(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), WorthQueryHomeRetirementDenial> {
        self.source_owner.preflight_home_retirement(request)
    }
}
