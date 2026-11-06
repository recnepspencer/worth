use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectKey;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::LocatorAuthority;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::installed_layout;

#[test]
fn installed_completion_layout_preserves_the_causal_record_fields() {
    let installed = installed_layout();
    let layout = installed.provider_inbound_completion();
    let expected = |field: &str| {
        AspectFieldLocator::new(
            LocatorAuthority::Planned,
            AspectKey::new("inbound-completion").unwrap(),
            CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
        )
    };
    assert_eq!(layout.correlation, expected("correlation"));
    assert_eq!(layout.family, expected("family"));
    assert_eq!(layout.operation, expected("operation"));
    assert_eq!(layout.audience, expected("audience"));
    assert_eq!(layout.source, expected("source"));
    assert_eq!(layout.expires_at, expected("expires-at"));
    assert_eq!(layout.key_epoch, expected("key-epoch"));
    assert_eq!(layout.message, expected("message"));
    assert_eq!(layout.protocol, expected("protocol"));
    assert_eq!(layout.version, expected("version"));
    assert_eq!(layout.meaning_digest, expected("meaning-digest"));
    assert_eq!(layout.payload, expected("payload"));
    assert_eq!(layout.original_commit, expected("original-commit"));
    assert_eq!(layout.original_branch, expected("original-branch"));
    assert_eq!(layout.original_entity, expected("original-entity"));
    assert_eq!(
        layout.original_incarnation,
        expected("original-incarnation")
    );
    assert_eq!(layout.terminal, expected("terminal"));
    assert_eq!(layout.provenance_kind, expected("provenance-kind"));
    assert_eq!(layout.transport_attempt, expected("transport-attempt"));
    assert_eq!(
        layout.transport_observation,
        expected("transport-observation")
    );
}
