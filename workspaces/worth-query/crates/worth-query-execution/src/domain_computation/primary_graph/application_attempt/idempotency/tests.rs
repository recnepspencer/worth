use super::{
    WorthQueryApplicationIdempotencyBinding, WorthQueryIdempotencyEntityIdentity,
    WorthQueryIdempotencyScopeIdentity,
};

#[test]
fn governed_proposal_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let first = baseline.bind_governed_proposal(Some(&[3; 32]));
    let retry = baseline.bind_governed_proposal(Some(&[3; 32]));
    let drift = baseline.bind_governed_proposal(Some(&[4; 32]));

    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert_eq!(first.key_text(), drift.key_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}

#[test]
fn governed_input_is_a_private_part_of_idempotency_intent() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let first = baseline.bind_governed_input(Some(&[3; 32]));
    let retry = baseline.bind_governed_input(Some(&[3; 32]));
    let drift = baseline.bind_governed_input(Some(&[4; 32]));

    assert_eq!(first.key_text(), retry.key_text());
    assert_eq!(first.intent_text(), retry.intent_text());
    assert_eq!(first.key_text(), drift.key_text());
    assert_ne!(first.intent_text(), drift.intent_text());
}

#[test]
fn private_identity_slots_cannot_alias_each_other() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let identity = [3; 32];
    let precondition = baseline.bind_preconditions(Some(&identity)).intent_text();
    let input = baseline.bind_governed_input(Some(&identity)).intent_text();
    let proposal = baseline
        .bind_governed_proposal(Some(&identity))
        .intent_text();

    assert_ne!(precondition, input);
    assert_ne!(precondition, proposal);
    assert_ne!(input, proposal);
}

#[test]
fn admitted_principal_and_scope_are_distinct_idempotency_components() {
    let baseline = WorthQueryApplicationIdempotencyBinding::new([1; 32], [2; 32]);
    let mut first = baseline;
    first.operation_scope_identity = Some(scope_identity(10, 20));
    let mut principal_drift = baseline;
    principal_drift.operation_scope_identity = Some(scope_identity(11, 20));
    let mut scope_drift = baseline;
    scope_drift.operation_scope_identity = Some(scope_identity(10, 21));

    assert_ne!(first.intent_text(), principal_drift.intent_text());
    assert_ne!(first.intent_text(), scope_drift.intent_text());
}

fn scope_identity(principal_slot: u64, scope_slot: u64) -> WorthQueryIdempotencyScopeIdentity {
    WorthQueryIdempotencyScopeIdentity {
        runtime_authority: 3,
        binding_runtime: 4,
        binding_generation: 5,
        package_identity: [6; 32],
        schema_identity: [7; 32],
        principal: WorthQueryIdempotencyEntityIdentity {
            partition: 8,
            local_slot: principal_slot,
            generation: 9,
        },
        scope: WorthQueryIdempotencyEntityIdentity {
            partition: 8,
            local_slot: scope_slot,
            generation: 9,
        },
    }
}
