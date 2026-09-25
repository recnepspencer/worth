use super::*;

#[test]
fn complete_comparable_fact_kinds_round_trip_without_losing_absence_or_sets() {
    let entity = EntityId::new(worth_relational::facade::identity::PartitionId(3), 9, 2);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("test.aspect").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("optional").unwrap()),
    );
    let facts = vec![
        Fact::SourceEntity { entity_id: entity },
        Fact::SourceAspectRevision {
            entity_id: entity,
            aspect: AspectKey::new("test.aspect").unwrap(),
            native_revision: Some(11),
        },
        Fact::SourceFieldRevision {
            entity_id: entity,
            locator,
            native_revision: Some(RelationalFieldRevision::new(
                VersionId(12),
                RelationalFieldPresence::Absent,
            )),
        },
        Fact::SourceAdjacencyRevision {
            relation_kind: KindId(13),
            anchor: entity,
            direction: RelationalAdjacencyDirection::Incoming,
            native_revision: None,
            comparison_work_limit: 20,
            endpoints: vec![entity],
        },
        Fact::Entity {
            entity_id: entity,
            kind: KindId(14),
        },
    ];
    let encoded = encode(&facts).expect("all facts have native comparison meaning");
    assert_eq!(decode(&encoded).unwrap().as_ref(), facts.as_slice());
}

#[test]
fn incomplete_or_unsupported_facts_never_gain_checkpoint_reuse_payload() {
    let entity = EntityId::new(worth_relational::facade::identity::PartitionId(0), 1, 1);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("test.aspect").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("optional").unwrap()),
    );
    assert!(encode(&[]).is_none());
    assert!(encode(&[Fact::SourceFieldRevision {
        entity_id: entity,
        locator: locator.clone(),
        native_revision: None,
    }])
    .is_none());
    assert!(encode(&[Fact::AbsentField {
        entity_id: entity,
        kind: KindId(1),
        locator,
    }])
    .is_none());
}

#[test]
fn fact_decode_rejects_unbounded_count_and_foreign_kind_before_allocation() {
    assert!(decode(&u32::MAX.to_be_bytes()).is_err());
    assert!(decode(&[0, 0, 0, 1, 255]).is_err());
    assert!(decode(&vec![0; MAXIMUM_FACT_BYTES + 1]).is_err());
}
