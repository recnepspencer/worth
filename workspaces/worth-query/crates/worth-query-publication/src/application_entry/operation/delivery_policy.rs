//! Optional caller-selected work restrictions for one native delivery attempt.
use std::num::NonZeroUsize;

/// Omitted work restrictions add no ceiling; native physical admission still applies.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorthQueryOperationDeliveryPolicy {
    pub(in crate::application_entry) currentness_work: Option<NonZeroUsize>,
    pub(super) batch_limits:
        Option<worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryBatchLimits>,
    pub(in crate::application_entry) query_limits: Option<(NonZeroUsize, NonZeroUsize)>,
}
impl WorthQueryOperationDeliveryPolicy {
    /// Narrows currentness work; any explicit native installed ceiling still applies.
    pub const fn currentness(maximum_work: NonZeroUsize) -> Self {
        Self {
            currentness_work: Some(maximum_work),
            query_limits: None,
            batch_limits: None,
        }
    }
    /// Narrows the installed aggregate batch work and custody allowance.
    pub const fn batch(
        mut self,
        limits: worth_query_execution::facade::primary_graph::WorthQueryApplicationQueryBatchLimits,
    ) -> Self {
        self.batch_limits = Some(limits);
        self
    }
    /// Requests narrower declared-query result and work ceilings.
    pub const fn query(
        mut self,
        maximum_results: NonZeroUsize,
        maximum_work: NonZeroUsize,
    ) -> Self {
        self.query_limits = Some((maximum_results, maximum_work));
        self
    }
}
