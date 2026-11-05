//! Host lease requests borrow the one execution authority installed in World.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::execution_memory::WorthQueryExecutionLeaseDenial;
use worth_execution::{ExecutionResourceLease, LeaseRequest};
use worth_query_installation::facade::ApplicationSchema;

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Requests the caller's explicit policy from the host authority in World.
    ///
    /// Omission returns [`WorthQueryExecutionLeaseDenial::Unavailable`]; Query
    /// never constructs a pool or supplies an unticketed alternative. The lease
    /// grants resource admission only, not graph or operation authority. Its
    /// memory tickets may outlive both this borrow and the originating World.
    /// Captured graph allocations are not automatically charged by this API.
    pub fn request_execution_lease(
        &self,
        request: LeaseRequest,
    ) -> Result<ExecutionResourceLease<'_>, WorthQueryExecutionLeaseDenial> {
        self.runtime
            .product_runtime
            .owner
            .execution_authority()
            .ok_or(WorthQueryExecutionLeaseDenial::Unavailable)?
            .request_lease(request)
            .map_err(WorthQueryExecutionLeaseDenial::Lease)
    }
}
