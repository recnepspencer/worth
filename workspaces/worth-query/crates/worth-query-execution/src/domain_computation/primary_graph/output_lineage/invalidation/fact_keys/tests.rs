use std::{cell::Cell, sync::Arc};

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalF64, CanonicalFieldPath, FieldKey,
    LocatorAuthority,
};
use worth_relational::facade::{
    history::RelationalDescriptiveTouch,
    identity::{EntityId, KindId, PartitionId},
    indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind},
    runtime::RelationalFieldPresence,
    storage::AuthoritativeFieldComparisonKey,
    transactions::RecordRef,
};

use super::{visit, Fact, FactKeyProjectionStop, FullVerificationReason, Key};

fn locator() -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("account").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("status").unwrap()),
    )
}

#[test]
fn absent_field_and_native_revision_share_exact_addresses_after_admission() {
    let entity = EntityId::new(PartitionId::main(), 7, 1);
    let kind = KindId(3);
    let locator = locator();
    let fact = Fact::AbsentField {
        entity_id: entity,
        kind,
        locator: locator.clone(),
    };
    let admitted = Cell::new(0usize);
    let emitted = Cell::new(0usize);
    let mut fact_keys = Vec::new();
    visit(
        &fact,
        |_, _| {
            admitted.set(admitted.get() + 1);
            Ok::<_, ()>(())
        },
        |key| {
            assert!(admitted.get() > emitted.get());
            emitted.set(emitted.get() + 1);
            fact_keys.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();

    let touch = RelationalDescriptiveTouch::FieldRevision {
        record: RecordRef::Entity(entity),
        kind,
        aspect: locator.aspect().aspect_key().clone(),
        path: locator.field_path().clone(),
        presence: RelationalFieldPresence::Absent,
    };
    let mut changed_keys = Vec::new();
    super::super::touch_keys::visit(
        &touch,
        |_| Ok::<_, ()>(()),
        |key| {
            changed_keys.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert!(changed_keys.iter().any(|key| fact_keys.contains(key)));
    assert!(fact_keys.contains(&Key::EntityLifecycle(entity)));
    assert!(fact_keys.iter().any(
        |key| matches!(key, Key::FieldRevision { entity: observed, .. } if *observed == entity)
    ));
}

#[test]
fn sibling_field_revision_does_not_match_an_unmodified_field() {
    let entity = EntityId::new(PartitionId::main(), 7, 1);
    let kind = KindId(3);
    let first = locator();
    let second = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        first.aspect().aspect_key().clone(),
        CanonicalFieldPath::single(FieldKey::new("owner").unwrap()),
    );
    let keys_for = |locator: AspectFieldLocator| {
        let mut keys = Vec::new();
        visit(
            &Fact::AbsentField {
                entity_id: entity,
                kind,
                locator,
            },
            |_, _| Ok::<_, ()>(()),
            |key| {
                keys.push(key);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        keys
    };
    let first_keys = keys_for(first);
    let second_keys = keys_for(second.clone());
    let touch = RelationalDescriptiveTouch::FieldRevision {
        record: RecordRef::Entity(entity),
        kind,
        aspect: second.aspect().aspect_key().clone(),
        path: second.field_path().clone(),
        presence: RelationalFieldPresence::Absent,
    };
    let mut touched = Vec::new();
    super::super::touch_keys::visit(
        &touch,
        |_| Ok::<_, ()>(()),
        |key| {
            touched.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert!(touched.iter().any(|key| second_keys.contains(key)));
    let aspect_touch = RelationalDescriptiveTouch::AspectRevision {
        record: RecordRef::Entity(entity),
        aspect: second.aspect().aspect_key().clone(),
    };
    super::super::touch_keys::visit(
        &aspect_touch,
        |_| Ok::<_, ()>(()),
        |key| {
            touched.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert!(touched.iter().all(|key| !first_keys.contains(key)));
}

#[test]
fn indexed_selection_matches_old_key_and_definition_touches() {
    let index = DerivedIndexId(9);
    let kind = KindId(3);
    let locator = locator();
    let value = AspectValue::Float64(CanonicalF64::from_f64(-0.0));
    let fact = Fact::IndexedEntitySelection {
        index_id: index,
        definition: Arc::new(DerivedIndexDefinition {
            index_id: index,
            name: "account-status".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: locator.clone(),
            },
            branch_scoped: true,
        }),
        entity_kind: kind,
        locator: locator.clone(),
        value: value.clone(),
        candidate_limit: 4,
        candidates: Vec::new(),
    };
    let mut fact_keys = Vec::new();
    visit(
        &fact,
        |_, _| Ok::<_, ()>(()),
        |key| {
            fact_keys.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    let touch = RelationalDescriptiveTouch::IndexMembership {
        index,
        kind,
        aspect: locator.aspect().aspect_key().clone(),
        path: locator.field_path().clone(),
        old_key: Some(AuthoritativeFieldComparisonKey::from_aspect_value(&value)),
        new_key: None,
    };
    let mut touched = Vec::new();
    super::super::touch_keys::visit(
        &touch,
        |_| Ok::<_, ()>(()),
        |key| {
            touched.push(key);
            Ok::<_, ()>(())
        },
    )
    .unwrap();
    assert!(touched.iter().any(|key| fact_keys.contains(key)));
    assert!(fact_keys.contains(&Key::IndexDefinition(index)));
    let indexed = fact_keys
        .iter()
        .find(|key| matches!(key, Key::IndexMembership { .. }))
        .unwrap();
    let expected_payload = locator
        .aspect()
        .aspect_key()
        .owned_allocation_capacity_bytes()
        + locator.field_path().owned_allocation_capacity_bytes()
        + AuthoritativeFieldComparisonKey::from_aspect_value(&value)
            .owned_allocation_capacity_bytes() as usize;
    assert_eq!(
        indexed.owned_payload_capacity_bytes(),
        Some(expected_payload as u64)
    );
}

#[test]
fn entity_local_field_and_anchor_postings_do_not_collide_across_unrelated_entities() {
    let mut field_keys = Vec::new();
    let mut adjacency_keys = Vec::new();
    for ordinal in [1, 10, 100] {
        let entity = EntityId::new(PartitionId::main(), ordinal, 1);
        let field = Fact::AbsentField {
            entity_id: entity,
            kind: KindId(3),
            locator: locator(),
        };
        let adjacency = Fact::Adjacency {
            relation_kind: KindId(5),
            anchor: entity,
            direction: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAdjacencyDirection::Outgoing,
            maximum_work_units: 8,
            relations: Vec::new(),
        };
        let mut keys = Vec::new();
        visit(
            &field,
            |_, _| Ok::<_, ()>(()),
            |key| {
                keys.push(key);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        field_keys.push(keys);
        let mut keys = Vec::new();
        visit(
            &adjacency,
            |_, _| Ok::<_, ()>(()),
            |key| {
                keys.push(key);
                Ok::<_, ()>(())
            },
        )
        .unwrap();
        adjacency_keys.push(keys);
    }
    for collection in [field_keys, adjacency_keys] {
        for left in 0..collection.len() {
            for right in left + 1..collection.len() {
                assert!(collection[left]
                    .iter()
                    .all(|key| !collection[right].contains(key)));
            }
        }
    }
}

#[test]
fn workflow_definition_and_capacity_facts_require_typed_full_verification() {
    let lineage = EntityId::new(PartitionId::main(), 7, 1);
    let definition = EntityId::new(PartitionId::main(), 8, 1);
    let facts = [
        Fact::WorkflowDefinitionPredecessor {
            relation_kind: KindId(5),
            lineage: Some(lineage),
            expected_definition: Some(definition),
            maximum_work_units: 8,
        },
        Fact::WorkflowDefinitionCurrent {
            relation_kind: KindId(5),
            lineage,
            expected_definition: definition,
            maximum_work_units: 8,
        },
        Fact::WorkflowInstanceCapacity {
            relation_kind: KindId(6),
            lineage,
            maximum_instances: 2,
            instances: Vec::new(),
        },
    ];
    for fact in facts {
        let emitted = Cell::new(0_usize);
        assert!(matches!(
            visit(
                &fact,
                |_, _| Ok::<_, ()>(()),
                |_| {
                    emitted.set(emitted.get() + 1);
                    Ok::<_, ()>(())
                }
            ),
            Err(FactKeyProjectionStop::FullVerificationRequired(
                FullVerificationReason::UnsupportedFact
            ))
        ));
        assert_eq!(emitted.get(), 0, "no posting names a workflow fact");
    }
}
