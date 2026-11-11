//! Public original producer attribution survives real schema-history recovery.
use super::branch_history_recovery::{checkpoint, current_head, recover};
use super::commit_meaning::{assert_durable_commit_meaning_eq, durable_commit_meaning_matches};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, RelationalRuntimeProfile},
    history::BranchId,
    runtime::RelationalRuntimeApi,
};
use worth_testkit_relational::{
    aspect_storage::AspectSchemaFixture, initial_input_source::schema_transition_history,
};

#[test]
fn original_input_observation_preserves_complete_attribution_across_schema_checkpoint_recovery() {
    let host=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &worth_execution::ExecutionRequest::serial(&host);
    let (source, medium) = schema_transition_history(request, 8, 2);
    let head = current_head(&source, &BranchId("main".into())).unwrap();
    let original = source
        .replay()
        .canonical_commit_envelope(request, head.commit_id)
        .unwrap()
        .unwrap();
    let predecessor = source
        .replay()
        .canonical_commit_envelope(request, head.parents[0])
        .unwrap()
        .unwrap();
    let observation = original
        .original_input_observation()
        .expect("actual original producer inputs");
    let previous = predecessor
        .original_input_observation()
        .expect("actual preceding original inputs");
    assert_ne!(observation.prestate_schema(), observation.result_schema());
    assert_ne!(observation, previous);
    let introduced = observation
        .introduced_schema()
        .expect("real schema carrier transition");
    assert_eq!(introduced.carrier_digest(), observation.result_schema());
    let debug = format!("{introduced:?}");
    assert!(!debug.contains("_memory"));
    assert!(!debug.contains("SchemaImageData"));
    assert!(!debug.contains("entity_registrations"));
    checkpoint(request, &source, &medium);
    let registry = AspectSchemaFixture {
        relation_kind_name: "s".repeat(8),
        ..AspectSchemaFixture::with_default_declared_aspects(
            CascadeDeletePolicy::CascadeDeleteRelations,
        )
    }
    .build_registry();
    let receiver = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build_durable(medium.attachment())
        .unwrap();
    let (_, receiver) = recover(request, &medium, receiver);
    let restored = receiver
        .replay()
        .canonical_commit_envelope(request, head.commit_id)
        .unwrap()
        .unwrap();
    let recovered = restored
        .original_input_observation()
        .expect("original attribution recovered");
    assert_eq!(observation, recovered);
    let recovered_schema = recovered.introduced_schema().unwrap();
    assert_eq!(
        introduced.carrier_digest(),
        recovered_schema.carrier_digest()
    );
    assert_eq!(introduced.schema_root(), recovered_schema.schema_root());
    assert_eq!(
        introduced.descriptor_semantics_version(),
        recovered_schema.descriptor_semantics_version()
    );
    assert_durable_commit_meaning_eq(&original, &restored);
    assert!(durable_commit_meaning_matches(&original, &restored));
    assert!(!durable_commit_meaning_matches(&predecessor, &restored));
}
