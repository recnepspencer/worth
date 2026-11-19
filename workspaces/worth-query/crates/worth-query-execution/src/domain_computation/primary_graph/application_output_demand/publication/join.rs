//! Exact owner/occurrence join; data equality grants no delivery authority.
use super::*;
impl RecoveredProducerOutputFacts {
    pub(in crate::domain_computation::primary_graph) fn matches_original(
        &self,
        original: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> bool {
        use worth_foundational::facade::{
            RecordedCompositePublicationAttemptIdentity, RecordedProductBranchReference,
        };
        let publication = &self.publication;
        let owner = publication.product_branch().owner_identity();
        let same_head = match original.new_product_head() {
            RecordedProductBranchReference::Home(head) => {
                owner.durable() == Some(head.branch().owner())
                    && head.branch().name() == publication.product_branch().name().as_str()
                    && head.incarnation().ordinal() == publication.product_incarnation().ordinal()
                    && head.generation().get() == publication.product_generation().get()
                    && head.selected_commit().ordinal() == publication.composite_commit().ordinal()
            }
            RecordedProductBranchReference::Memory(head) => {
                owner.durable().is_none()
                    && head.owner() == owner.get()
                    && head.branch() == publication.product_branch().name().as_str()
                    && head.incarnation() == publication.product_incarnation().ordinal()
                    && head.generation() == publication.product_generation().get()
                    && head.selected_commit() == publication.composite_commit().ordinal()
            }
        };
        let attempt = publication.publication_attempt();
        let same_attempt = match original.attempt_identity() {
            RecordedCompositePublicationAttemptIdentity::Home(value) => {
                attempt.owner_identity().durable() == Some(value.owner())
                    && attempt.ordinal() == value.ordinal()
            }
            RecordedCompositePublicationAttemptIdentity::Memory { owner, ordinal } => {
                attempt.owner_identity().durable().is_none()
                    && attempt.owner_identity().get() == owner
                    && attempt.ordinal() == ordinal
            }
        };
        let same_settlement = match (original.exact_output_settlement(), self.settlement.as_ref()) {
            (worth_foundational::facade::DurableQueryOutputSettlement::Absent, None) => true,
            (
                worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot {
                    slot,
                },
                Some(identity),
            ) => {
                let (occurrence, generation, actual_slot) = identity.address();
                occurrence == publication.product_incarnation()
                    && generation == publication.product_generation().get()
                    && u64::try_from(actual_slot).ok() == Some(slot)
            }
            (worth_foundational::facade::DurableQueryOutputSettlement::Absent, Some(_))
            | (
                worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot {
                    ..
                },
                None,
            ) => false,
        };
        same_head
            && same_attempt
            && same_settlement
            && self.outcome == original.outcome_identity()
            && publication.relational_commit() == original.commit_reference()
            && self.changes.commit_reference() == original.commit_reference()
            && self.authority.idempotency_binding().key_identity()
                == original.idempotency_key_identity()
    }
}
