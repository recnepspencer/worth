//! Query retains authored field observations while Native admits unique writes.
use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectContractRevision, AspectFieldLocator, AspectIdentity, AspectKey, AspectValue,
    CanonicalFieldPath, FieldKey, InternedString, LocatorAuthority, PortableAspectContractBasis,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};

use super::{prepare_provider_attempt, WorthQueryApplicationRealizedEffect as Effect};
use crate::domain_computation::primary_graph::application_attempt::effect_program::{
    WorthQueryApplicationCreationPartition as Partition, WorthQueryApplicationOptionalFieldWrite,
};
use crate::domain_computation::primary_graph::application_attempt::{
    CompletedFactRoutes, CompletedReadScopes, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind as Kind, WorthQueryApplicationObservedFact as Fact,
};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use crate::domain_computation::primary_graph::StorageControl;

const KIND: KindId = KindId(7);
#[test]
fn creates_reach_native_validation_without_an_implicit_key_read() {
    for partition in [Partition::Issued, Partition::Context(PartitionId(5))] {
        assert_eq!(lower(vec![], vec![create("a", "ada", partition)]), Ok(()));
    }
}

#[test]
fn an_existing_write_keeps_its_original_field_observation() {
    assert_eq!(lower(vec![held(1, "bob")], vec![update(1, "ada")]), Ok(()));
    assert!(lower(vec![], vec![update(1, "ada")]).is_err());
}

#[test]
fn clearing_a_unique_field_needs_no_selection_of_its_old_value() {
    assert_eq!(lower(vec![held(1, "ada")], vec![patch(1, None)]), Ok(()));
}

#[test]
fn a_nonunique_field_lowers_without_a_key_observation() {
    let effect = Effect::CreateEntity {
        kind: KIND,
        key: "a".to_owned(),
        fields: BTreeMap::from([(field("other"), text("ada"))]),
        partition: Partition::Issued,
    };
    assert_eq!(lower(vec![], vec![effect]), Ok(()));
}

#[test]
fn other_lowering_denials_keep_their_kind_and_subject_at_the_commit_boundary() {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
        WorthQueryApplicationCommitDenialStage,
    };
    let attempt = WorthQueryApplicationAttemptDenial::new(
        Kind::IncompleteEffectBasis,
        "provider progression",
    );
    let denial = WorthQueryApplicationCommitDenial::effect_lowering_denied(&attempt);
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProposalBinding
    );
    assert_eq!(
        denial.detail(),
        Some("IncompleteEffectBasis: provider progression")
    );
}

fn lower(facts: Vec<Fact>, effects: Vec<Effect>) -> Result<(), Kind> {
    let policy = worth_execution::ExecutionAllocationPolicy::SystemAllocation;
    let facts = super::super::read_set::admit_array(
        facts.len(),
        facts,
        policy,
        "unique fixture",
        || Ok(()),
    )
    .unwrap();
    let routes = CompletedFactRoutes::from_facts(
        FactPayload::native(facts),
        StorageControl::new(policy, None),
    )
    .unwrap();
    prepare_provider_attempt(
        &|_, _| None,
        PartitionId::main(),
        effects.len(),
        CompletedReadScopes::EmptyOwnerFixture,
        routes,
        None,
        None,
        Vec::new(),
        effects,
        None,
        None,
        Default::default(),
        false,
        false,
        &[],
        None,
    )
    .map(|_| ())
    .map_err(|denial| denial.kind())
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
