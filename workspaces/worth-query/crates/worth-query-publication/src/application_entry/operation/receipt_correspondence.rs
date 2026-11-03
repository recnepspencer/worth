//! Pair original data with an already-owned live receipt; never construct one.
use worth_query_execution::facade::primary_graph::{
    RecordedProductBranchReference, WorthQueryApplicationCommitReceipt,
    WorthQueryRecoveredApplicationCommitReceipt,
};
pub(super) fn matches_retained_original(
    live: &WorthQueryApplicationCommitReceipt,
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    let original = live.committed_product_publication();
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
        && original.publication_attempt().ordinal() == recorded.attempt_identity().ordinal()
        && live.commit_reference() == recorded.commit_reference()
        && live.idempotency_binding().key_identity() == recorded.idempotency_key_identity()
        && live.outcome_identity() == Some(recorded.outcome_identity())
        && live.emitted_effect_count() == recorded.emitted_effect_count()
        && live.recorded_output_settlement() == recorded.exact_output_settlement()
}

/// Canonical subject commitments avoid traversing unbounded semantic payloads
/// merely to confirm an already-held recorded continuation.
pub(super) fn same_recorded_subject(
    left: &WorthQueryRecoveredApplicationCommitReceipt,
    right: &WorthQueryRecoveredApplicationCommitReceipt,
) -> bool {
    left.commit_reference() == right.commit_reference()
        && left.new_product_head() == right.new_product_head()
        && left.old_product_head() == right.old_product_head()
        && left.attempt_identity() == right.attempt_identity()
        && left.native_journal_position() == right.native_journal_position()
        && left.native_artifact_digest() == right.native_artifact_digest()
        && left.world_journal_position() == right.world_journal_position()
        && left.world_artifact_digest() == right.world_artifact_digest()
        && left.publishing_incarnation() == right.publishing_incarnation()
        && left.idempotency_key_identity() == right.idempotency_key_identity()
        && left.outcome_identity() == right.outcome_identity()
        && left.emitted_effect_count() == right.emitted_effect_count()
        && left.exact_output_settlement() == right.exact_output_settlement()
}
