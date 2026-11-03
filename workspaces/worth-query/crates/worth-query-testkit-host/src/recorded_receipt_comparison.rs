//! Compare durable receipt meaning with an independently retained original.
use worth_query_host::facade::primary_graph::{
    WorldOwnerCoordinate, WorthQueryApplicationCommitReceipt,
    WorthQueryRecoveredApplicationCommitReceipt,
};

pub fn assert_original(
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
    original: &WorthQueryApplicationCommitReceipt,
) {
    assert_eq!(recorded.commit_reference(), original.commit_reference());
    assert_eq!(
        Some(recorded.outcome_identity()),
        original.outcome_identity()
    );
    assert_eq!(
        recorded.idempotency_key_identity(),
        original.idempotency_binding().key_identity()
    );
    assert_eq!(
        recorded.changed_record_count(),
        original.changed_record_count()
    );
    assert_eq!(
        recorded.emitted_effect_count(),
        original.emitted_effect_count()
    );
    assert_eq!(
        recorded.exact_output_settlement(),
        original.recorded_output_settlement()
    );
    let publication = original.committed_product_publication();
    let head = recorded
        .new_product_head()
        .memory()
        .expect("the court owns an explicit memory World");
    assert_eq!(
        head.owner(),
        publication.product_branch().owner_identity().get()
    );
    assert_eq!(head.branch(), publication.product_branch().name().as_str());
    assert_eq!(
        head.incarnation(),
        publication.product_incarnation().ordinal()
    );
    assert_eq!(head.generation(), publication.product_generation().get());
    assert_eq!(
        head.selected_commit(),
        publication.composite_commit().ordinal()
    );
    assert_eq!(
        recorded.attempt_identity().ordinal(),
        publication.publication_attempt().ordinal()
    );
    assert_eq!(
        recorded.attempt_identity().owner(),
        WorldOwnerCoordinate::MemoryOnly {
            observation: publication.publication_attempt().owner_identity().get(),
        }
    );
    assert!(recorded.publishing_incarnation().is_none());
    assert!(recorded.world_journal_position().is_none());
    assert!(recorded.world_artifact_digest().is_none());
}

/// Recovered data identifies the exact original transition; it cannot perform it.
pub fn assert_transition(
    recorded: &WorthQueryRecoveredApplicationCommitReceipt,
    original: &worth_query_host::facade::application_entry::PerformedWorkflowTransition,
) {
    assert_original(recorded, original.receipt());
    assert!(
        recorded
            .semantic_result()
            .semantic()
            .created_entities()
            .iter()
            .any(|row| row.assigned == original.transition()),
        "the original artifact assigns the same transition entity"
    );
}
