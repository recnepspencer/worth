use worth_foundational::facade::AspectValue;
    use std::collections::BTreeMap;
    use worth_relational::facade::identity::EntityId;
    use worth_foundational::facade::InternedString;
    use worth_query_execution::domain_computation::primary_graph::workflow::assessment::dependency::append_field_revision_fields;
    use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_instance_observation::settlement::dependency::decode_field_revision_fact;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_foundational::facade::AspectFieldLocator;
    use worth_foundational::facade::CanonicalFieldPath;
    use worth_foundational::facade::LocatorAuthority;
    use worth_relational::facade::identity::PartitionId;
    use worth_relational::facade::identity::VersionId;
    use worth_relational::facade::runtime::RelationalFieldPresence;
    use worth_relational::facade::runtime::RelationalFieldRevision;

#[test]
fn durable_field_dependency_roundtrips_presence_and_denies_partial_revision() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let layout = &world
        .application
        .runtime
        .primary_graph()
        .unwrap()
        .layout
        .workflow()
        .evidence_dependency;
    let entity = EntityId::new(PartitionId::main(), 17, 2);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        worth_foundational::facade::AspectKey::new("account-status").unwrap(),
        CanonicalFieldPath::single(worth_foundational::facade::FieldKey::new("status").unwrap()),
    );
    for presence in [
        RelationalFieldPresence::Present,
        RelationalFieldPresence::Absent,
    ] {
        let revision = RelationalFieldRevision::new(VersionId(42), presence);
        let mut encoded = BTreeMap::new();
        append_field_revision_fields(layout, &locator, Some(revision), &mut encoded);
        let string = |field| match encoded.get(field).unwrap() {
            AspectValue::String(InternedString::Raw(value)) => value.clone(),
            _ => panic!("string evidence field"),
        };
        let version = match encoded.get(&layout.native_revision).unwrap() {
            AspectValue::UInt64(value) => *value,
            _ => panic!("revision evidence field"),
        };
        let decoded = decode_field_revision_fact(
            entity,
            Some(string(&layout.aspect)),
            Some(string(&layout.field)),
            Some(version),
            Some(string(&layout.field_presence)),
        )
        .unwrap();
        assert_eq!(decoded, worth_query_execution::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id: entity, locator: locator.clone(), native_revision: Some(revision),
            });
        assert!(decode_field_revision_fact(
            entity,
            Some(string(&layout.aspect)),
            Some(string(&layout.field)),
            Some(version),
            None
        )
        .is_err());
    }
    let mut unavailable = BTreeMap::new();
    append_field_revision_fields(layout, &locator, None, &mut unavailable);
    assert!(!unavailable.contains_key(&layout.native_revision));
    assert!(!unavailable.contains_key(&layout.field_presence));
}
