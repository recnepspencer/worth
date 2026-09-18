use worth_foundational::facade::{
    AspectContractRevision, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue,
    CanonicalFieldPath, FieldKey, LocatorAuthority, PortableAspectContractBasis,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, RelationId};

use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

use super::super::effect_program::WorthQueryApplicationOptionalFieldWrite;
use super::super::effect_program::WorthQueryApplicationRealizedEffect;
use super::*;

fn dependency_identity(
    declared_key: [u8; 32],
    facts: &[WorthQueryApplicationObservedFact],
) -> Result<
    (
        [u8; 32],
        worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ),
    (),
> {
    super::dependency_identity(declared_key, facts, 4 * 1_024 * 1_024)
}

#[test]
fn producer_dependency_identity_rejects_an_insufficient_host_byte_budget() {
    let fact = WorthQueryApplicationObservedFact::SourceEntity {
        entity_id: EntityId::new(PartitionId::main(), 41, 1),
    };
    assert!(super::dependency_identity([9; 32], &[fact.clone()], 1).is_err());
    assert!(super::dependency_identity([9; 32], &[fact], 4 * 1_024).is_ok());
}

#[test]
fn producer_identity_changes_with_any_completed_dependency() {
    let entity_id = EntityId::new(PartitionId::main(), 17, 2);
    let fact = |revision| WorthQueryApplicationObservedFact::SourceAspectRevision {
        entity_id,
        aspect: AspectKey::new("structural-facts").unwrap(),
        native_revision: Some(revision),
    };

    let first = dependency_identity([9; 32], &[fact(41)]).unwrap();
    let retry = dependency_identity([9; 32], &[fact(41)]).unwrap();
    let changed_fact = dependency_identity([9; 32], &[fact(42)]).unwrap();
    let changed_declared_key = dependency_identity([10; 32], &[fact(41)]).unwrap();

    assert_eq!(first.0, retry.0);
    assert_eq!(first.1, retry.1);
    assert_ne!(first.0, changed_fact.0);
    assert_ne!(first.0, changed_declared_key.0);
    assert_eq!(first.1.digest_derivations(), 1);
}

#[test]
fn field_presence_values_and_relation_sets_are_distinct_dependencies() {
    let entity_id = EntityId::new(PartitionId::main(), 17, 2);
    let other = EntityId::new(PartitionId::main(), 18, 2);
    let kind = KindId::new(4);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("structural-facts").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("axis").unwrap()),
    );
    let field = |value| WorthQueryApplicationObservedFact::Field {
        entity_id,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(value),
    };
    let absent = WorthQueryApplicationObservedFact::AbsentField {
        entity_id,
        kind,
        locator: locator.clone(),
    };
    let relation = |slot| WorthQueryApplicationObservedFact::Relation {
        relation_kind: KindId::new(5),
        from: entity_id,
        to: other,
        matching_relations: vec![RelationId::new(PartitionId::main(), slot, 1)],
    };

    assert_ne!(identity(field(1)), identity(field(2)));
    assert_ne!(identity(field(1)), identity(absent));
    assert_ne!(identity(relation(21)), identity(relation(22)));
}

#[test]
fn dependency_identity_budget_covers_the_admitted_dependency_set() {
    let facts = (0..4_096)
        .map(|slot| WorthQueryApplicationObservedFact::SourceEntity {
            entity_id: EntityId::new(PartitionId::main(), slot, 1),
        })
        .collect::<Vec<_>>();

    let (_, work) = dependency_identity([7; 32], &facts)
        .expect("the digest budget is derived from the admitted dependency set");
    assert_eq!(work.digest_derivations(), 1);
}

#[test]
fn self_written_field_uses_its_postcondition_for_retry_identity() {
    let entity_id = EntityId::new(PartitionId::main(), 17, 2);
    let kind = KindId::new(4);
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("structural-facts").unwrap(),
        CanonicalFieldPath::single(FieldKey::new("axis").unwrap()),
    );
    let fact = |value| WorthQueryApplicationObservedFact::Field {
        entity_id,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(value),
    };
    let effect = WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "body".to_owned(),
        entity_id,
        fields: std::collections::BTreeMap::from([(locator.clone(), AspectValue::UInt64(2))]),
    };

    let first_facts = normalized_output_facts(&[fact(1)], &[effect]);
    let first = dependency_identity([4; 32], &first_facts).unwrap().0;
    let retry = dependency_identity([4; 32], &[fact(2)]).unwrap().0;

    assert_eq!(first, retry);
}

#[test]
fn writing_one_field_keeps_the_source_aspect_revision_dependency() {
    let entity_id = EntityId::new(PartitionId::main(), 17, 2);
    let aspect = AspectKey::new("structural-facts").unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect.clone(),
        CanonicalFieldPath::single(FieldKey::new("axis").unwrap()),
    );
    let source_aspect = WorthQueryApplicationObservedFact::SourceAspectRevision {
        entity_id,
        aspect,
        native_revision: Some(41),
    };
    let effect = WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: "body".to_owned(),
        entity_id,
        fields: std::collections::BTreeMap::from([(locator, AspectValue::UInt64(2))]),
    };

    let normalized = normalized_output_facts(&[source_aspect.clone()], &[effect]);

    assert_eq!(normalized, vec![source_aspect]);
}

#[test]
fn optional_field_postconditions_preserve_presence_and_absence() {
    let entity_id = EntityId::new(PartitionId::main(), 17, 2);
    let kind = KindId::new(4);
    let aspect = AspectKey::new("structural-facts").unwrap();
    let locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect.clone(),
        CanonicalFieldPath::single(FieldKey::new("axis").unwrap()),
    );
    let contract = || {
        PortableAspectContractBasis::new(
            aspect.clone(),
            AspectIdentity(11),
            AspectContractRevision(3),
        )
    };
    let patch = |value| WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
        entity: "body".to_owned(),
        entity_id,
        fields: std::collections::BTreeMap::from([(
            locator.clone(),
            WorthQueryApplicationOptionalFieldWrite {
                contract: contract(),
                value,
            },
        )]),
    };
    let present = WorthQueryApplicationObservedFact::Field {
        entity_id,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(1),
    };
    let absent = WorthQueryApplicationObservedFact::AbsentField {
        entity_id,
        kind,
        locator: locator.clone(),
    };

    assert_eq!(
        normalized_output_facts(&[present], &[patch(None)]),
        vec![absent.clone()]
    );
    assert_eq!(
        normalized_output_facts(&[absent], &[patch(Some(AspectValue::UInt64(2)))]),
        vec![WorthQueryApplicationObservedFact::Field {
            entity_id,
            kind,
            locator,
            value: AspectValue::UInt64(2),
        }]
    );
}

#[test]
fn lineage_identity_converges_on_retry_and_breaks_aba_replay() {
    let dependency_a = [1; 32];
    let dependency_b = [2; 32];
    let first_a = lineage_identity(dependency_a, None).unwrap().0;
    let retry_a = lineage_identity(dependency_a, None).unwrap().0;
    let first_b = lineage_identity(dependency_b, Some(retry_a)).unwrap().0;
    let second_a = lineage_identity(dependency_a, Some(first_b)).unwrap().0;
    let retry_second_a = lineage_identity(dependency_a, Some(first_b)).unwrap().0;

    assert_eq!(first_a, retry_a);
    assert_ne!(first_b, first_a);
    assert_ne!(second_a, first_a);
    assert_ne!(second_a, first_b);
    assert_eq!(second_a, retry_second_a);
}

fn identity(fact: WorthQueryApplicationObservedFact) -> [u8; 32] {
    dependency_identity([3; 32], &[fact]).unwrap().0
}
