use super::super::ResourceRuntimeState;
use crate::branch::owner_services::conditional_execution::SignalOwnedAsyncRetentionStorageForTest;

impl ResourceRuntimeState {
    pub(in crate::logic::transaction::runtime) fn retention_storage_for_test(
        &self,
        retired_wakes: usize,
    ) -> SignalOwnedAsyncRetentionStorageForTest {
        SignalOwnedAsyncRetentionStorageForTest {
            retired_wakes,
            in_flight: self.in_flight_by_request.len(),
            terminal: self.terminal_in_flight_by_request.len(),
            retained_lifecycle: self.retained_in_flight_history_by_request.len(),
            pruned_lifecycle: self.pruned_in_flight_history_by_request.len(),
            denied: self.denied_completions.len(),
            pruned_denied: self.pruned_denied_completions_by_id.len(),
            latest_denied: self.latest_denied_completion_by_node.len(),
            retained_retry: self.retained_retry_lineage_by_ordinal.len(),
            pruned_retry: self.pruned_retry_lineage_by_ordinal.len(),
            pending_retry_requests: self.pending_retry_by_request.len(),
            pending_retry_wakes: self.pending_retry_by_wake.len(),
            pending_retry_nodes: self.pending_retry_by_node.len(),
            retry_budget_generations: self.retry_budget_ledger.request_generation_count_for_test(),
        }
    }
}
