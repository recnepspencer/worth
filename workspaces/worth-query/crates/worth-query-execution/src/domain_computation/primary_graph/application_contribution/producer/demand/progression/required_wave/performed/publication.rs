//! The authoritative commit fact seals the wave member before optional work.
use super::{FreshReadiness, PerformedMembers};
use crate::domain_computation::primary_graph::application_contribution::producer::execution::ProducerCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;

impl PerformedMembers {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn publication(
        &mut self,
        readiness: FreshReadiness,
        receipt: ProducerCommitReceipt,
    ) -> WorthQueryApplicationCommitReceipt {
        receipt.record_publication(|| self.performed(readiness))
    }
}
