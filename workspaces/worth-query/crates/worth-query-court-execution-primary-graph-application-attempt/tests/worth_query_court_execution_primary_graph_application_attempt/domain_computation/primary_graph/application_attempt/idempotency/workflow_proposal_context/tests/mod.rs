use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;

#[test]
fn workflow_proposal_context_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let first = baseline.bind_workflow_proposal_context(&[3; 32]);
    let retry = baseline.bind_workflow_proposal_context(&[3; 32]);
    let drift = baseline.bind_workflow_proposal_context(&[4; 32]);

    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert_eq!(first.key_text(), drift.key_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}
