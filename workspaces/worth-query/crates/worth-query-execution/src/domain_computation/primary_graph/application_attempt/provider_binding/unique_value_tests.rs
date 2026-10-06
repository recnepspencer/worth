//! The unique-value lowering law over sealed facts: a write of a unique field
//! lowers only beside the field's observed indexed selection of that value,
//! and only when that selection holds no live entity but the one written.

use std::collections::BTreeMap;
use std::sync::Arc;

use worth_foundational::facade::{
    AspectContractRevision, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue,
    CanonicalFieldPath, FieldKey, InternedString, LocatorAuthority, PortableAspectContractBasis,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::indexes::{DerivedIndexDefinition, DerivedIndexId, DerivedIndexKind};

use super::{prepare_provider_attempt, WorthQueryApplicationRealizedEffect as Effect};
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    WorthQueryApplicationCreationPartition as Partition, WorthQueryApplicationOptionalFieldWrite,
    WorthQueryCandidateValidatorWorkAdmission,
};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind as Kind,
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFieldFixture;

const KIND: KindId = KindId(7);
const INDEX: DerivedIndexId = DerivedIndexId(8);

#[test]
fn a_create_of_a_free_value_lowers_in_both_partitions() {
    for partition in [Partition::Issued, Partition::Context(PartitionId(5))] {
        let facts = vec![selection("ada", vec![])];
        assert_eq!(
            lower(Some(INDEX), facts, vec![create("a", "ada", partition)]),
            Ok(())
        );
    }
}

#[test]
fn a_create_of_a_taken_value_is_denied_in_both_partitions() {
    for partition in [Partition::Issued, Partition::Context(PartitionId(5))] {
        let facts = vec![selection("ada", vec![entity(1)])];
        assert_eq!(
            lower(Some(INDEX), facts, vec![create("a", "ada", partition)]),
            Err(Kind::UniqueValueTaken)
        );
    }
}

#[test]
fn a_unique_write_without_its_observed_selection_does_not_lower() {
    assert_eq!(
        lower(
            Some(INDEX),
            vec![],
            vec![create("a", "ada", Partition::Issued)]
        ),
        Err(Kind::IncompleteEffectBasis)
    );
    assert_eq!(
        lower(Some(INDEX), vec![held(1, "bob")], vec![update(1, "ada")]),
        Err(Kind::IncompleteEffectBasis)
    );
}

#[test]
fn one_program_writes_each_unique_value_at_most_once() {
    let twice = vec![
        create("a", "ada", Partition::Issued),
        create("b", "ada", Partition::Issued),
    ];
    assert_eq!(
        lower(Some(INDEX), vec![selection("ada", vec![])], twice),
        Err(Kind::UniqueValueTaken)
    );
    let facts = vec![held(1, "bob"), selection("ada", vec![])];
    let update_then_create = vec![update(1, "ada"), create("a", "ada", Partition::Issued)];
    assert_eq!(
        lower(Some(INDEX), facts, update_then_create),
        Err(Kind::UniqueValueTaken)
    );
}

#[test]
fn an_update_or_patch_into_a_taken_value_is_denied() {
    let facts = || vec![held(1, "bob"), selection("ada", vec![entity(2)])];
    assert_eq!(
        lower(Some(INDEX), facts(), vec![update(1, "ada")]),
        Err(Kind::UniqueValueTaken)
    );
    assert_eq!(
        lower(Some(INDEX), facts(), vec![patch(1, Some("ada"))]),
        Err(Kind::UniqueValueTaken)
    );
}

#[test]
fn a_write_that_keeps_its_own_value_lowers() {
    let facts = || vec![held(1, "ada"), selection("ada", vec![entity(1)])];
    assert_eq!(lower(Some(INDEX), facts(), vec![update(1, "ada")]), Ok(()));
    assert_eq!(
        lower(Some(INDEX), facts(), vec![patch(1, Some("ada"))]),
        Ok(())
    );
}

#[test]
fn clearing_a_unique_field_frees_its_value_without_a_lookup() {
    assert_eq!(
        lower(Some(INDEX), vec![held(1, "ada")], vec![patch(1, None)]),
        Ok(())
    );
}

#[test]
fn deleting_the_holder_does_not_free_its_value_in_the_same_program() {
    let facts = vec![
        Fact::Entity {
            entity_id: entity(1),
            kind: KIND,
        },
        selection("ada", vec![entity(1)]),
    ];
    let effects = vec![
        Effect::DeleteEntity {
            entity_id: entity(1),
        },
        create("a", "ada", Partition::Issued),
    ];
    assert_eq!(
        lower(Some(INDEX), facts, effects),
        Err(Kind::UniqueValueTaken)
    );
}

#[test]
fn an_uninstalled_index_is_its_own_denial() {
    let facts = vec![selection("ada", vec![])];
    assert_eq!(
        lower(None, facts, vec![create("a", "ada", Partition::Issued)]),
        Err(Kind::UniqueIndexUnavailable)
    );
    assert_eq!(
        lower(None, vec![held(1, "bob")], vec![update(1, "ada")]),
        Err(Kind::UniqueIndexUnavailable)
    );
}

#[test]
fn restored_duplicates_deny_writes_of_their_value_only() {
    let facts = || {
        vec![
            held(1, "ada"),
            selection("ada", vec![entity(1), entity(2)]),
            selection("bob", vec![]),
        ]
    };
    assert_eq!(
        lower(Some(INDEX), facts(), vec![update(1, "ada")]),
        Err(Kind::UniqueValueTaken)
    );
    assert_eq!(lower(Some(INDEX), facts(), vec![update(1, "bob")]), Ok(()));
    assert_eq!(
        lower(
            Some(INDEX),
            facts(),
            vec![create("b", "bob", Partition::Issued)]
        ),
        Ok(())
    );
}

#[test]
fn a_field_that_is_not_unique_needs_no_selection() {
    let effect = Effect::CreateEntity {
        kind: KIND,
        key: "a".to_owned(),
        fields: BTreeMap::from([(field("other"), text("ada"))]),
        partition: Partition::Issued,
    };
    assert_eq!(lower(Some(INDEX), vec![], vec![effect]), Ok(()));
}

#[test]
fn unique_lowering_denials_keep_their_kind_at_the_commit_boundary() {
    for (kind, expected) in [
        (
            Kind::UniqueValueTaken,
            WorthQueryApplicationCommitDenialKind::UniqueValueTaken,
        ),
        (
            Kind::UniqueIndexUnavailable,
            WorthQueryApplicationCommitDenialKind::UniqueIndexUnavailable,
        ),
    ] {
        let attempt = WorthQueryApplicationAttemptDenial::new(kind, "Key");
        let denial = WorthQueryApplicationCommitDenial::effect_lowering_denied(&attempt);
        assert_eq!(denial.kind(), expected);
        assert_eq!(
            denial.stage(),
            WorthQueryApplicationCommitDenialStage::EffectLowering
        );
        assert_eq!(denial.detail(), Some("Key"));
    }
}

fn lower(
    index: Option<DerivedIndexId>,
    facts: Vec<Fact>,
    effects: Vec<Effect>,
) -> Result<(), Kind> {
    let fixture = WorthQueryUniqueFieldFixture::new(KIND, field("key"), index);
    prepare_provider_attempt(
        fixture.fields(),
        PartitionId::main(),
        effects.len(),
        Vec::new(),
        facts,
        Vec::new(),
        effects,
        0,
        0,
        None,
        None,
        WorthQueryCandidateValidatorWorkAdmission::unreserved_internal(),
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .map(|_| ())
    .map_err(|denial| denial.kind())
}

fn selection(value: &str, candidates: Vec<EntityId>) -> Fact {
    Fact::IndexedEntitySelection {
        index_id: INDEX,
        definition: Arc::new(DerivedIndexDefinition {
            index_id: INDEX,
            name: "unique-key".to_owned(),
            kind: DerivedIndexKind::EntityField {
                field_locator: field("key"),
            },
            branch_scoped: true,
        }),
        entity_kind: KIND,
        locator: field("key"),
        value: text(value),
        candidate_limit: 2,
        candidates,
    }
}

fn held(slot: u64, value: &str) -> Fact {
    Fact::Field {
        entity_id: entity(slot),
        kind: KIND,
        locator: field("key"),
        value: text(value),
    }
}

fn create(key: &str, value: &str, partition: Partition) -> Effect {
    Effect::CreateEntity {
        kind: KIND,
        key: key.to_owned(),
        fields: BTreeMap::from([(field("key"), text(value))]),
        partition,
    }
}

fn update(slot: u64, value: &str) -> Effect {
    Effect::UpdateEntity {
        entity: "Unique".to_owned(),
        entity_id: entity(slot),
        fields: BTreeMap::from([(field("key"), text(value))]),
    }
}

fn patch(slot: u64, value: Option<&str>) -> Effect {
    Effect::PatchOptionalEntityFields {
        entity: "Unique".to_owned(),
        entity_id: entity(slot),
        fields: BTreeMap::from([(
            field("key"),
            WorthQueryApplicationOptionalFieldWrite {
                contract: PortableAspectContractBasis::new(
                    AspectKey::new("identity").unwrap(),
                    AspectIdentity(11),
                    AspectContractRevision(1),
                ),
                value: value.map(text),
            },
        )]),
    }
}

fn field(name: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new("identity").unwrap(),
        CanonicalFieldPath::single(FieldKey::new(name).unwrap()),
    )
}

fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 0)
}
