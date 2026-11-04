//! What a committed application reports to the advancement observer.
use super::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::request_execution::advancement::observation::application::WorthQueryObservedCommit;

impl WorthQueryObservedCommit for WorthQueryApplicationCommitReceipt {
    fn observed_operation(&self) -> [u8; 32] {
        *self.installed_operation()
    }
    fn observed_idempotency_key(&self) -> [u8; 32] {
        *self.idempotency_binding().key_identity()
    }
    fn observed_idempotency_intent(&self) -> [u8; 32] {
        *self.idempotency_binding().intent_identity()
    }
}
