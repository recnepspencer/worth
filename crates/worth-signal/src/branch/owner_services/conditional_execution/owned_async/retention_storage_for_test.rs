use super::*;
use crate::data::temporal::TemporalWakeSummary;

/// Physical container widths in the actual selected conditional cell. These
/// diagnostic counts carry no lifecycle or source authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignalOwnedAsyncRetentionStorageForTest {
    pub in_flight: usize,
    pub terminal: usize,
    pub retained_lifecycle: usize,
    pub pruned_lifecycle: usize,
    pub denied: usize,
    pub pruned_denied: usize,
    pub latest_denied: usize,
    pub retained_retry: usize,
    pub pruned_retry: usize,
    pub pending_retry_requests: usize,
    pub pending_retry_wakes: usize,
    pub pending_retry_nodes: usize,
    pub retry_budget_generations: usize,
    pub retired_wakes: usize,
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    #[doc(hidden)]
    pub fn owned_async_retention_storage_for_test(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
    ) -> Result<
        (SignalOwnedAsyncRetentionStorageForTest, TemporalWakeSummary),
        SignalConditionalServiceExecutionDenial,
    > {
        self.require_owned_async_binding(binding)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.conditional_owned_async_retention_storage_for_test(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
            )
        })
    }
}
