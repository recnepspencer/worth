use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;

#[test]
fn workflow_transition_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let first = baseline.bind_workflow_transition(&[3; 32]);
    let retry = baseline.bind_workflow_transition(&[3; 32]);
    let drift = baseline.bind_workflow_transition(&[4; 32]);

    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert_eq!(first.key_text(), drift.key_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}

#[test]
fn condition_source_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_workflow_transition(&[3; 32]);
    let first = baseline.bind_workflow_condition(&[4; 32]);
    let retry = baseline.bind_workflow_condition(&[4; 32]);
    let drift = baseline.bind_workflow_condition(&[5; 32]);

    assert_eq!(first.intent_text(), retry.intent_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}

#[test]
fn guarded_effect_has_one_transition_key_but_preserves_client_key_drift() {
    let first = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_guarded_workflow_effect(&[3; 32]);
    let retry = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_guarded_workflow_effect(&[3; 32]);
    let different_client_key = WorthQueryApplicationIdempotencyBinding::new([4; 32], [2; 32])
        .bind_guarded_workflow_effect(&[3; 32]);
    let different_transition = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
        .bind_guarded_workflow_effect(&[5; 32]);
    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert!(first.matches_guarded_workflow_effect(&[3; 32]));
    assert!(
        !WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32])
            .bind_workflow_operation(&[3; 32])
            .matches_guarded_workflow_effect(&[3; 32])
    );
    assert_eq!(first.key_text(), different_client_key.key_text());
    assert_ne!(first.intent_text(), different_client_key.intent_text());
    assert_ne!(first.key_text(), different_transition.key_text());
}
