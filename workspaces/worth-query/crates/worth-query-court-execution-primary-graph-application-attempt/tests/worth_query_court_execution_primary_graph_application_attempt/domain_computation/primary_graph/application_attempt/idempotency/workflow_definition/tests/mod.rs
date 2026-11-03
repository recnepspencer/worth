use worth_query_execution::facade::domain_computation::WorthQueryApplicationIdempotencyBinding;

#[test]
fn workflow_definition_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let first = baseline.bind_workflow_definition(&[3; 32]);
    let retry = baseline.bind_workflow_definition(&[3; 32]);
    let drift = baseline.bind_workflow_definition(&[4; 32]);

    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert_eq!(first.key_text(), drift.key_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}

#[test]
fn ordinary_intent_retains_its_pre_workflow_encoding() {
    let ordinary = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);

    assert_eq!(
        ordinary.intent_text(),
        concat!(
            "v2:0202020202020202020202020202020202020202020202020202020202020202",
            ":source=-:operation=-:scope=-:precondition=-:input=-:proposal=-",
            ":conditional-definition=-"
        )
    );
}

#[test]
fn workflow_identity_survives_combined_composition() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let combined = baseline
        .bind_operation(&[3; 32])
        .bind_preconditions(Some(&[4; 32]))
        .bind_governed_input(Some(&[5; 32]))
        .bind_governed_proposal(Some(&[6; 32]))
        .bind_workflow_definition(&[8; 32]);

    for drift in [
        baseline
            .bind_operation(&[7; 32])
            .bind_preconditions(Some(&[4; 32]))
            .bind_governed_input(Some(&[5; 32]))
            .bind_governed_proposal(Some(&[6; 32]))
            .bind_workflow_definition(&[8; 32]),
        combined.bind_preconditions(Some(&[7; 32])),
        combined.bind_governed_input(Some(&[7; 32])),
        combined.bind_governed_proposal(Some(&[7; 32])),
        combined.bind_workflow_definition(&[7; 32]),
    ] {
        assert_ne!(combined.intent_text(), drift.intent_text());
    }
}
