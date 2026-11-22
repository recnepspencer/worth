//! Crate-boundary port for internal `WorthQueryExecutionRuntime` transitions.
//! Never exported on `worth-query-execution::facade` or Host.

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceAdmissionDenial;

use crate::domain_computation::{
    WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime,
};

pub trait ExecutionRuntimeInternalPort {
    fn primary_graph_any(&self) -> Option<&(dyn std::any::Any + Send + Sync)>;
    fn primary_graph_any_mut(&mut self) -> Option<&mut (dyn std::any::Any + Send + Sync)>;
    fn install_primary_graph_any(&mut self, graph: Box<dyn std::any::Any + Send + Sync>);
    fn start_reserved_direct_resource_attempt(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        resources: worth_query_admission::integration::WorthQueryCapacityReservedExecutionResourcePlan,
    ) -> Result<WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionResourceAdmissionDenial>;
}

impl ExecutionRuntimeInternalPort for WorthQueryExecutionRuntime {
    fn primary_graph_any(&self) -> Option<&(dyn std::any::Any + Send + Sync)> {
        Self::primary_graph_any(self)
    }

    fn primary_graph_any_mut(&mut self) -> Option<&mut (dyn std::any::Any + Send + Sync)> {
        Self::primary_graph_any_mut(self)
    }

    fn install_primary_graph_any(&mut self, graph: Box<dyn std::any::Any + Send + Sync>) {
        Self::install_primary_graph_any(self, graph)
    }

    fn start_reserved_direct_resource_attempt(
        &self,
        authority: &WorthQueryExecutionBoundOperationAuthority,
        resources: worth_query_admission::integration::WorthQueryCapacityReservedExecutionResourcePlan,
    ) -> Result<WorthQueryDirectExecutionResourceAttempt, WorthQueryExecutionResourceAdmissionDenial>
    {
        Self::start_reserved_direct_resource_attempt(self, authority, resources)
    }
}
