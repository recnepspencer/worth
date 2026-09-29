use super::super::fixture::{
    CapabilityTouchOperation, IdentityExecutionSchema, ProgramRequiredOperation,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

type Schema = IdentityExecutionSchema;

#[test]
fn one_key_and_intent_under_two_operations_never_replay_each_other() {
    let program_required = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
        Schema,
        ProgramRequiredOperation,
        _,
        _,
    >("shared-key", "shared-intent")
    .expect("the host commit encodes");
    let capability_touch = WorthQueryApplicationIdempotencyBinding::for_host_commit::<
        Schema,
        CapabilityTouchOperation,
        _,
        _,
    >("shared-key", "shared-intent")
    .expect("the host commit encodes");

    assert_ne!(
        program_required.key_identity(),
        capability_touch.key_identity(),
        "a key reused under another operation is another key"
    );
    assert_ne!(
        program_required.intent_identity(),
        capability_touch.intent_identity(),
        "the same intent under another operation is another intent"
    );
}

#[test]
fn a_host_commit_replays_under_its_operation_and_drifts_with_its_intent() {
    let commit = |key: &str, intent: &str| {
        WorthQueryApplicationIdempotencyBinding::for_host_commit::<
            Schema,
            ProgramRequiredOperation,
            _,
            _,
        >(key, intent)
        .expect("the host commit encodes")
    };
    let first = commit("key", "intent");

    assert_eq!(first, commit("key", "intent"));
    assert_eq!(
        first.key_identity(),
        commit("key", "changed").key_identity()
    );
    assert_ne!(
        first.intent_identity(),
        commit("key", "changed").intent_identity()
    );
    assert_ne!(
        commit("same", "same").key_identity(),
        commit("same", "same").intent_identity(),
        "one value as key and as intent never shares an identity"
    );
}
