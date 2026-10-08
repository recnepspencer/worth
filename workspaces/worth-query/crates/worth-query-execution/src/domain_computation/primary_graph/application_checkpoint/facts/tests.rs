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
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(vec![entity]), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
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
    assert_eq!(
        decode(
            &encoded,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .unwrap()
        .as_ref(),
        facts.as_slice()
    );
    assert!(
        decode_for_wire_version(
            &encoded,
            5,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .is_err(),
        "v5 cannot admit a v6 retirement fact"
    );
    // One decode path: facts at an older wire version are never read, with
    // or without the newer fact kinds.
    assert!(decode_for_wire_version(
        &encode(&facts[..5]).unwrap(),
        5,
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation
    )
    .is_err());
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
    assert!(decode(
        &u32::MAX.to_be_bytes(),
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation
    )
    .is_err());
    assert!(decode(
        &[0, 0, 0, 1, 255],
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation
    )
    .is_err());
    assert!(decode(
        &vec![0; MAXIMUM_FACT_BYTES + 1],
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation
    )
    .is_err());
}

#[test]
fn complete_large_decisions_round_trip_above_the_former_total_fact_ceiling() {
    let facts = (1..=65_537)
        .map(|slot| Fact::SourceEntity {
            entity_id: EntityId::new(
                worth_relational::facade::identity::PartitionId(0),
                slot as u64,
                1,
            ),
        })
        .collect::<Vec<_>>();
    let encoded = encode(&facts).expect("complete decisions retain every dependency");
    assert_eq!(&encoded[..4], &65_537u32.to_be_bytes());
    assert_eq!(encoded.len(), 4 + 65_537 * 17);
    assert_eq!(
        decode(
            &encoded,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .unwrap()
        .as_ref(),
        facts.as_slice()
    );
    let mut forged = encoded;
    forged[..4].copy_from_slice(&65_538u32.to_be_bytes());
    assert!(decode(
        &forged,
        None,
        worth_execution::ExecutionAllocationPolicy::SystemAllocation
    )
    .is_err());
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
        endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(vec![entity; MAXIMUM_SET_ENTITIES + 1]), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
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

#[test]
fn optional_absence_is_exact_and_an_older_wire_version_is_never_read() {
    let entity = EntityId::new(worth_relational::facade::identity::PartitionId(1), 2, 1);
    let fact = Fact::SourceAspectRevision {
        entity_id: entity,
        aspect: AspectKey::new("test.optional").unwrap(),
        native_revision: None,
    };
    let bytes = encode(std::slice::from_ref(&fact)).expect("fresh absence has a bounded wire");
    assert_eq!(
        decode_for_wire_version(
            &bytes,
            WIRE_VERSION,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .unwrap()
        .as_ref(),
        &[fact]
    );
    for older in [5, 6] {
        assert!(decode_for_wire_version(
            &bytes,
            older,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .is_err());
    }
}

#[test]
fn indexed_selection_wire_preserves_semantic_definition_and_signed_zero() {
    use std::sync::Arc;
    use worth_foundational::facade::{AspectValue, CanonicalF64};
    use worth_relational::facade::indexes::{
        DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind,
    };

    let entity = EntityId::new(worth_relational::facade::identity::PartitionId(1), 2, 1);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        AspectKey::new("test.index").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("value").unwrap()),
    );
    let fact = Fact::IndexedEntitySelection {
        index_id: DerivedIndexId(8),
        definition: Arc::new(DerivedIndexDefinition {
            index_id: DerivedIndexId(8),
            name: "test-index".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: true,
        }),
        entity_kind: KindId(3),
        locator,
        value: AspectValue::Float64(CanonicalF64::from_f64(-0.0)),
        candidate_limit: 4,
        candidates: vec![entity],
    };
    let bytes = encode(std::slice::from_ref(&fact)).expect("bounded native index fact encodes");
    assert_eq!(
        decode_for_wire_version(
            &bytes,
            WIRE_VERSION,
            None,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation
        )
        .unwrap()
        .as_ref(),
        &[fact]
    );
}
