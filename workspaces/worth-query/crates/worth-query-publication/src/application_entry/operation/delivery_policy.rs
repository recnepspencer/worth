//! Caller-selected semantic result cardinality for one delivery.
use std::num::NonZeroUsize;
/// Caller-selected result cardinality for a published operation's queries.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorthQueryOperationDeliveryPolicy {
    pub(in crate::application_entry) query_limits: Option<NonZeroUsize>,
}
impl WorthQueryOperationDeliveryPolicy {
    pub const fn query(mut self, maximum_results: NonZeroUsize) -> Self {
        self.query_limits = Some(maximum_results);
        self
    }
}
