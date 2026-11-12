use super::temporal_idempotency::{prepare_temporal_idempotency, TemporalIdempotencyScope};
mod durable_identity;
use super::*;
use crate::domain_computation::primary_graph::conditional_operation::canonical_identity::{
    prepare_temporal_binding_identity, prepare_temporal_runtime_binding_identity,
    TemporalBindingIdentityParts, TemporalRuntimeBindingIdentityParts,
    WorthQueryTemporalRuntimeBindingIdentity,
};
use worth_foundational::facade::CanonicalDigestId;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_installation::facade::{
    WorthQueryClockCoordinate, WorthQueryTemporalIntentIdempotencyRelation,
    WorthQueryTemporalIntentIdentity, WorthQueryTemporalOperationInputIdentity,
};

struct TestClock;

fn candidate(input_identity: &str) -> WorthQueryTemporalIntentCandidate<TestClock, String> {
    WorthQueryTemporalIntentCandidate::active::<StringApplicationValueBinding>(
        WorthQueryTemporalIntentIdentity::declare("intent-7").unwrap(),
        "record-7".to_string(),
        4,
        WorthQueryClockCoordinate::from_nanoseconds(90),
        "operation-input".to_string(),
        WorthQueryTemporalOperationInputIdentity::declare(input_identity).unwrap(),
        WorthQueryTemporalIntentIdempotencyRelation::declare(&"relation-7").unwrap(),
    )
    .expect("fixture record identity must encode")
}

fn runtime_binding(seed: u8) -> WorthQueryTemporalRuntimeBindingIdentity {
    runtime_binding_with_live(seed, "node-authority", 11)
}
fn runtime_binding_with_live(
    seed: u8,
    authority: &str,
    live: u64,
) -> WorthQueryTemporalRuntimeBindingIdentity {
    runtime_binding_with_node(seed, seed, authority, live)
}
fn runtime_binding_with_node(
    query: u8,
    node: u8,
    authority: &str,
    live: u64,
) -> WorthQueryTemporalRuntimeBindingIdentity {
    let binding = prepare_temporal_binding_identity(TemporalBindingIdentityParts {
        node_authority: authority,
        node_definition: (
            CanonicalDigestId::new([node; 32]),
            worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
        ),
        clock: "clock",
        source: "source",
        timeline: "timeline",
        query: CanonicalDigestId::new([query; 32]),
        projector: "projector",
        principal_source: "principal-source",
        invoker: "invoker",
    })
    .unwrap();
    prepare_temporal_runtime_binding_identity(TemporalRuntimeBindingIdentityParts {
        binding: &binding,
        runtime_authority: live,
        installation_runtime: live + 1,
        installation_generation: live + 2,
        provider: "provider",
        branch: "main",
    })
    .unwrap()
}

#[test]
fn temporal_idempotency_binds_exact_operation_input_meaning() {
    let binding = runtime_binding(1);
    let first = prepare_temporal_idempotency(
        &binding,
        &candidate("input-a"),
        TemporalIdempotencyScope::ProcessLocal,
    )
    .unwrap()
    .binding();
    let duplicate = prepare_temporal_idempotency(
        &binding,
        &candidate("input-a"),
        TemporalIdempotencyScope::ProcessLocal,
    )
    .unwrap()
    .binding();
    let changed_input = prepare_temporal_idempotency(
        &binding,
        &candidate("input-b"),
        TemporalIdempotencyScope::ProcessLocal,
    )
    .unwrap()
    .binding();

    assert_eq!(first.key_identity(), duplicate.key_identity());
    assert_eq!(first.intent_identity(), duplicate.intent_identity());
    assert_eq!(first.key_identity(), changed_input.key_identity());
    assert_ne!(first.intent_identity(), changed_input.intent_identity());
}

#[test]
fn temporal_idempotency_is_exact_binding_affine() {
    let first = prepare_temporal_idempotency(
        &runtime_binding(1),
        &candidate("input-a"),
        TemporalIdempotencyScope::ProcessLocal,
    )
    .unwrap()
    .binding();
    let foreign = prepare_temporal_idempotency(
        &runtime_binding(2),
        &candidate("input-a"),
        TemporalIdempotencyScope::ProcessLocal,
    )
    .unwrap()
    .binding();

    assert_ne!(first.key_identity(), foreign.key_identity());
    assert_eq!(first.intent_identity(), foreign.intent_identity());
}
