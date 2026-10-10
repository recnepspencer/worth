//! The authoritative commit fact seals the wave member before optional work.
use super::{PerformedMembers, PublicationReadiness};
use crate::domain_computation::primary_graph::application_contribution::producer::execution::ProducerCommitReceipt;

/// Only a newly committed effect lends this member's publication seal.
/// A replay never constructs this capability.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PublishedMember<
    'record,
> {
    performed: &'record mut bool,
}
impl PublishedMember<'_> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn performed(
        self,
    ) {
        *self.performed = true;
    }
}

/// The receipt and a read-only view of the record at its publication handoff.
pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct PublicationHandoff<
    'record,
> {
    receipt: ProducerCommitReceipt,
    recorded: &'record bool,
}
impl<'record> PublicationHandoff<'record> {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn into_parts(
        self,
    ) -> (ProducerCommitReceipt, &'record bool) {
        (self.receipt, self.recorded)
    }
}

impl PerformedMembers {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn publication<
        'record,
    >(
        &'record mut self,
        readiness: PublicationReadiness,
        receipt: ProducerCommitReceipt,
        mark: impl FnOnce(PublishedMember<'_>),
    ) -> PublicationHandoff<'record> {
        let performed = &mut self.entries[readiness.member].performed;
        let receipt = receipt.record_publication(|| mark(PublishedMember { performed }));
        PublicationHandoff {
            receipt,
            recorded: performed,
        }
    }
}
