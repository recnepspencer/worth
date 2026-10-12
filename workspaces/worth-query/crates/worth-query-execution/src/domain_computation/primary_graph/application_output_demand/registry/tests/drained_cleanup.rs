//! Released publication custody waits for the next selected, paying door.
use super::published_retirement::{admit, balances, publish};
use super::*;
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn fixture_released_publication_cleanup(
        &self,
        receipt: crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> (usize, usize, usize) {
        let baseline = balances(self);
        let stale_key = key("drained-cleanup-upstream", 1, 1);
        let newest_key = key("drained-cleanup-upstream", 2, 1);
        let dependent_key = key("drained-cleanup-owner", 1, 2);
        let stale = admit(self, &stale_key);
        let dependent = admit(self, &dependent_key);
        publish(self, &dependent, receipt.clone());
        {
            // Declare one funded consumed edge: its release is this fixture's subject.
            let mut state = self.state.lock().unwrap();
            let claims = vec![Arc::new(stale_key.clone())];
            state.required_reserved_bytes +=
                claims.capacity() * std::mem::size_of::<Arc<WorthQueryOutputDemandKey>>();
            state.records.get_mut(&dependent_key).unwrap().prerequisites = claims;
            state
                .records
                .get_mut(&stale_key)
                .unwrap()
                .framework_required_count += 1;
        }
        drop(stale);
        let newest = admit(self, &newest_key);
        publish(self, &newest, receipt);
        drop(newest);
        drop(dependent);
        assert!(
            self.fixture_cleanup_head().is_some(),
            "released custody is explicitly noted for a payer"
        );
        baseline
    }
    pub(in crate::domain_computation::primary_graph) fn fixture_cleanup_head(
        &self,
    ) -> Option<WorthQueryOutputDemandKey> {
        self.state
            .lock()
            .unwrap()
            .pending_cleanup_head
            .as_deref()
            .cloned()
    }
    pub(in crate::domain_computation::primary_graph) fn fixture_reserved_balances(
        &self,
    ) -> (usize, usize, usize) {
        balances(self)
    }
}
