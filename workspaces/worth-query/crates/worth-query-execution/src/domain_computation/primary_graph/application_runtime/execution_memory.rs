//! Host lease requests borrow the one execution authority installed in World.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_execution::{ExecutionResourceLease, LeaseDenial, LeaseRequest};
use worth_query_installation::facade::ApplicationSchema;

/// Refusal to admit explicit execution resources through this application's World.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryExecutionLeaseDenial {
    /// The host installed no process execution authority in this World.
    Unavailable,
    /// The installed execution authority refused the caller's lease request.
    Lease(LeaseDenial),
}

impl std::fmt::Display for WorthQueryExecutionLeaseDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("World has no installed execution authority"),
            Self::Lease(denial) => write!(formatter, "World execution lease refused: {denial:?}"),
        }
    }
}

impl std::error::Error for WorthQueryExecutionLeaseDenial {}

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
        self.product_runtime
            .owner
            .execution_authority()
            .ok_or(WorthQueryExecutionLeaseDenial::Unavailable)?
            .request_lease(request)
            .map_err(WorthQueryExecutionLeaseDenial::Lease)
    }
}
