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
        Fact::RetiredOutputEntity {
            entity_id: entity,
            kind: KindId(14),
            created_at: VersionId(5),
            deleted_at: VersionId(15),
            read_locator: "original-read-locator".into(),
        },
    ];
    let encoded = encode(&facts).expect("all facts have native comparison meaning");
    assert_eq!(decode(&encoded).unwrap().as_ref(), facts.as_slice());
    assert!(
        decode_version(&encoded, 5).is_err(),
        "v5 cannot admit a v6 retirement fact"
    );
    assert_eq!(
        decode_version(&encode(&facts[..5]).unwrap(), 5)
            .unwrap()
            .as_ref(),
        &facts[..5]
    );
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

#[test]
fn complete_large_decisions_round_trip_at_the_total_fact_ceiling() {
    let facts = (1..=MAXIMUM_FACTS)
        .map(|slot| Fact::SourceEntity {
            entity_id: EntityId::new(
                worth_relational::facade::identity::PartitionId(0),
                slot as u64,
                1,
            ),
        })
        .collect::<Vec<_>>();
    let encoded = encode(&facts).expect("bounded decisions retain every dependency");
    assert!(encoded.len() > 1024 * 1024);
    assert_eq!(decode(&encoded).unwrap().as_ref(), facts.as_slice());
    let mut excessive = facts;
    excessive.push(excessive[0].clone());
    assert!(encode(&excessive).is_none());
    let mut forged = encoded;
    forged[..4].copy_from_slice(&((MAXIMUM_FACTS + 1) as u32).to_be_bytes());
    assert!(decode(&forged).is_err());
}

#[test]
fn total_byte_capacity_does_not_widen_each_membership_set() {
    let entity = EntityId::new(worth_relational::facade::identity::PartitionId(0), 1, 1);
    let adjacency = Fact::SourceAdjacencyRevision {
        relation_kind: KindId(1),
        anchor: entity,
        direction: RelationalAdjacencyDirection::Outgoing,
        native_revision: Some(VersionId(1)),
        comparison_work_limit: MAXIMUM_SET_ENTITIES + 1,
        endpoints: vec![entity; MAXIMUM_SET_ENTITIES + 1],
    };
    assert!(encode(&[adjacency]).is_none());
    let large_fact = Fact::SourceAspectRevision {
        entity_id: entity,
        aspect: AspectKey::new("a".repeat(MAXIMUM_TEXT)).unwrap(),
        native_revision: Some(1),
    };
    // Count is admissible; complete encoding must still refuse excess bytes.
    assert!(encode(&vec![large_fact; MAXIMUM_FACT_BYTES / MAXIMUM_TEXT + 1]).is_none());
}
