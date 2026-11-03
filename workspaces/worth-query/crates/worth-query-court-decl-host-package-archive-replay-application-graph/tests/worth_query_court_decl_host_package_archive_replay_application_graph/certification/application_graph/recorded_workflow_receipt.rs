//! Retries compare owner-readmitted data with the genuine original effect.
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionPublicationOutcome, WorkflowDefinitionRetirementOutcome,
    WorkflowInstanceStartOutcome, WorkflowProposalOutcome,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt as Live,
    WorthQueryRecoveredApplicationCommitReceipt as Recorded,
};
pub(super) fn definition(outcome: WorkflowDefinitionPublicationOutcome) -> Recorded {
    match outcome {
        WorkflowDefinitionPublicationOutcome::RecoveredCommitted(value) => value,
        other => panic!("expected recorded definition: {other:?}"),
    }
}
pub(super) fn start(outcome: WorkflowInstanceStartOutcome) -> Recorded {
    match outcome {
        WorkflowInstanceStartOutcome::RecoveredCommitted(value) => value,
        other => panic!("expected recorded start: {other:?}"),
    }
}
pub(super) fn proposal(outcome: WorkflowProposalOutcome) -> Recorded {
    match outcome {
        WorkflowProposalOutcome::RecoveredCommitted(value) => value,
        other => panic!("expected recorded proposal: {other:?}"),
    }
}
pub(super) fn retirement(outcome: WorkflowDefinitionRetirementOutcome) -> Recorded {
    match outcome {
        WorkflowDefinitionRetirementOutcome::RecoveredCommitted(value) => value,
        other => panic!("expected recorded retirement: {other:?}"),
    }
}
pub(super) fn original(recorded: &Recorded, live: &Live) {
    assert_eq!(recorded.commit_reference(), live.commit_reference());
    assert_eq!(Some(recorded.outcome_identity()), live.outcome_identity());
    assert_eq!(recorded.changed_record_count(), live.changed_record_count());
    assert_eq!(recorded.emitted_effect_count(), live.emitted_effect_count());
    assert_eq!(
        recorded.idempotency_key_identity(),
        live.idempotency_binding().key_identity()
    );
    let head = recorded
        .new_product_head()
        .memory()
        .expect("this fixture is explicit memory-only");
    let publication = live.committed_product_publication();
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
}
pub(super) fn creates(recorded: &Recorded, entity: worth_relational::facade::identity::EntityId) {
    assert!(
        recorded
            .semantic_result()
            .semantic()
            .created_entities()
            .iter()
            .any(|row| row.assigned == entity),
        "the exact original entity correspondence survives retry"
    );
}
