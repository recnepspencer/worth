//! Descriptive correspondence never grants a live source or delivery witness.
use super::WorthQueryApplicationCommitReceipt;
use crate::domain_computation::primary_graph::WorthQueryRecoveredApplicationCommitReceipt;
use worth_foundational::facade::RecordedProductBranchReference;
impl WorthQueryApplicationCommitReceipt {
    pub fn matches_recorded_original(
        &self,
        recorded: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> bool {
        let original = self.committed_product_publication();
        let same_head = match recorded.new_product_head() {
            RecordedProductBranchReference::Home(head) => {
                original.product_branch().owner_identity().durable() == Some(head.branch().owner())
                    && original.product_branch().name().as_str() == head.branch().name()
                    && original.product_incarnation().ordinal() == head.incarnation().ordinal()
                    && original.product_generation().get() == head.generation().get()
                    && original.composite_commit().ordinal() == head.selected_commit().ordinal()
            }
            RecordedProductBranchReference::Memory(head) => {
                original
                    .product_branch()
                    .owner_identity()
                    .durable()
                    .is_none()
                    && original.product_branch().owner_identity().get() == head.owner()
                    && original.product_branch().name().as_str() == head.branch()
                    && original.product_incarnation().ordinal() == head.incarnation()
                    && original.product_generation().get() == head.generation()
                    && original.composite_commit().ordinal() == head.selected_commit()
            }
        };
        same_head
            && self
                .committed_changes
                .matches_recorded_semantic_result(recorded.semantic_result())
            && original.publication_attempt().ordinal() == recorded.attempt_identity().ordinal()
            && self.commit_reference() == recorded.commit_reference()
            && self.idempotency_binding().key_identity() == recorded.idempotency_key_identity()
            && self.outcome_identity() == Some(recorded.outcome_identity())
            && self.emitted_effect_count() == recorded.emitted_effect_count()
            && self.recorded_output_settlement() == recorded.exact_output_settlement()
    }
}
