use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;

#[test]
fn workflow_proposal_context_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new_for_test([1; 32], [2; 32]);
    let first = baseline.bind_workflow_proposal_context(&[3; 32]);
    let retry = baseline.bind_workflow_proposal_context(&[3; 32]);
    let drift = baseline.bind_workflow_proposal_context(&[4; 32]);

    assert_eq!(first.key_text_for_test(), retry.key_text_for_test());
    assert_eq!(first.intent_text_for_test(), retry.intent_text_for_test());
    assert_eq!(first.key_text_for_test(), drift.key_text_for_test());
    assert_ne!(first.intent_text_for_test(), drift.intent_text_for_test());
}
