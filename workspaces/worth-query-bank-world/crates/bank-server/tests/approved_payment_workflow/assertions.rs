use bank_domain::proposals::BankIdempotencyKey;
use worth_query_host::facade::application_entry::WorkflowProgressOutcome;

pub(super) fn require_assessment(outcome: WorkflowProgressOutcome, expected_node: &str) {
    match outcome {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), expected_node);
            assert_eq!(required.query(), "payment_detail");
        }
        other => panic!("expected assessment at {expected_node}, got {other:?}"),
    }
}

pub(super) fn require_completed(outcome: WorkflowProgressOutcome, expected_node: &str) {
    match outcome {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), expected_node);
        }
        other => panic!("expected completed {expected_node}, got {other:?}"),
    }
}

pub(super) fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}
