use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCreationPartition;
use worth_relational::facade::identity::{KindId, PartitionId};

#[test]
fn appended_workflow_facts_keep_their_context_after_an_issued_application_create() {
    let application_partition = PartitionId::new(3);
    let workflow_partition = PartitionId::new(7);
    let created = |partition_id, kind_id, key: &str| {
        EntityReference::Created(CreatedEntityRef {
            partition_id,
            kind_id: KindId::new(kind_id),
            client_key: worth_relational::facade::symbols::ClientKey::raw(key),
        })
    };
    let effects = vec![
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: KindId::new(10),
            key: "cad-feature".to_owned(),
            fields: BTreeMap::new(),
            partition: WorthQueryApplicationCreationPartition::Issued,
        },
        WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: KindId::new(11),
            key: "transition".to_owned(),
            fields: BTreeMap::new(),
            partition: WorthQueryApplicationCreationPartition::Context(workflow_partition),
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: KindId::new(12),
            key: "instance-transition".to_owned(),
            from: EntityReference::Existing(EntityId::new(workflow_partition, 1, 1)),
            to: created(workflow_partition, 11, "transition"),
        },
        WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: KindId::new(13),
            key: "cad-membership".to_owned(),
            from: EntityReference::Existing(EntityId::new(application_partition, 2, 1)),
            to: created(PartitionId::main(), 10, "cad-feature"),
        },
    ];
    let references = CreatedEffectReferences::new(&effects, application_partition);
    let facts = ObservedFactIndex::new(&[]);
    let mut unique = UniqueValueWrites::new(
        crate::domain_computation::primary_graph::schema_layout::WorthQueryUniqueFields::none(),
    );
    let mut lowered = effects.into_iter().map(|effect| {
        lower_provider_effect(
            &facts,
            &mut unique,
            &|_, _| None,
            &references,
            application_partition,
            effect,
        )
        .unwrap()
    });
    let entity_partition = |lowered| match lowered {
        WorthQueryLoweredProviderEffect::Mutation {
            intent: MutationIntent::Create(CreateIntent::Entity(entity)),
            ..
        } => entity.partition_id,
        _ => panic!("expected an entity create"),
    };
    assert_eq!(
        entity_partition(lowered.next().unwrap()),
        application_partition
    );
    assert_eq!(
        entity_partition(lowered.next().unwrap()),
        workflow_partition
    );
    let relation = |lowered| match lowered {
        WorthQueryLoweredProviderEffect::Mutation {
            intent: MutationIntent::Create(CreateIntent::Relation(relation)),
            ..
        } => relation,
        _ => panic!("expected a relation create"),
    };
    let workflow_relation = relation(lowered.next().unwrap());
    assert_eq!(workflow_relation.partition_id, workflow_partition);
    assert_eq!(workflow_relation.target.partition_id(), workflow_partition);
    let cad_relation = relation(lowered.next().unwrap());
    assert_eq!(cad_relation.partition_id, application_partition);
    assert_eq!(cad_relation.target.partition_id(), application_partition);
}
