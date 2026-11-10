use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::attachment_digest::SubscriptionConsumerAttachmentDigest;
use super::evidence_identities::{
    lifecycle_acknowledgement_frontier_identity, lifecycle_delivery_batch_receipt_identity,
};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct QueryDeliverySequence(u64);

impl QueryDeliverySequence {
    pub(super) fn initial() -> Self {
        Self(0)
    }

    pub(super) fn next(self) -> Self {
        Self(self.0 + 1)
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionAcknowledgementFrontier {
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    acknowledged_sequence: QueryDeliverySequence,
    frontier_identity: WorthQueryEvidenceIdentity,
}

impl SubscriptionAcknowledgementFrontier {
    pub(super) fn initial(attachment_digest: SubscriptionConsumerAttachmentDigest) -> Self {
        Self::new(attachment_digest, QueryDeliverySequence::initial())
    }

    pub(super) fn advance(&self, sequence: QueryDeliverySequence) -> Self {
        Self::new(self.attachment_digest.clone(), sequence)
    }

    fn new(
        attachment_digest: SubscriptionConsumerAttachmentDigest,
        acknowledged_sequence: QueryDeliverySequence,
    ) -> Self {
        let frontier_identity = lifecycle_acknowledgement_frontier_identity(
            attachment_digest.evidence_identity(),
            acknowledged_sequence.get(),
        );
        Self {
            attachment_digest,
            acknowledged_sequence,
            frontier_identity,
        }
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn acknowledged_sequence(&self) -> QueryDeliverySequence {
        self.acknowledged_sequence
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.frontier_identity
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryDeliveryBatchReceipt {
    attachment_digest: SubscriptionConsumerAttachmentDigest,
    sequence: QueryDeliverySequence,
    receipt_identity: WorthQueryEvidenceIdentity,
}

impl QueryDeliveryBatchReceipt {
    pub(super) fn new(
        attachment_digest: SubscriptionConsumerAttachmentDigest,
        sequence: QueryDeliverySequence,
    ) -> Self {
        let receipt_identity = lifecycle_delivery_batch_receipt_identity(
            attachment_digest.evidence_identity(),
            sequence.get(),
        );
        Self {
            attachment_digest,
            sequence,
            receipt_identity,
        }
    }

    pub(crate) fn attachment_digest(&self) -> &SubscriptionConsumerAttachmentDigest {
        &self.attachment_digest
    }

    pub fn sequence(&self) -> QueryDeliverySequence {
        self.sequence
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.receipt_identity
    }
}
