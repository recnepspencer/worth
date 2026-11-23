//! Fixture builders for the public restoration-header create/grow/retire/restore journey.
use worth_relational::facade::{
    identity::{EntityId, KindId, PartitionId},
    mvcc::*,
    symbols::ClientKey,
    transactions::*,
};
use worth_testkit_relational::aspect_storage::{
    aspect_key, field_key, publications::string_aspect_field_patch,
};

pub(super) fn target(key: impl Into<String>) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key.into()),
    }
}

pub(super) fn output_targets(route: usize, round: usize, members: usize) -> Vec<CreatedEntityRef> {
    (0..members)
        .map(|i| target(format!("route-{route}-round-{round}-member-{i}")))
        .collect()
}

pub(super) fn fields(aspect: &str, value: &str) -> AspectFieldPatch {
    string_aspect_field_patch([(aspect_key(aspect), field_key(aspect), value)])
}

pub(super) fn creates(targets: &[CreatedEntityRef]) -> WorkerIntentBatch {
    targets
        .iter()
        .fold(WorkerIntentBatch::new("authored-input"), |batch, t| {
            batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: t.partition_id,
                kind_id: t.kind_id,
                client_key: t.client_key.clone(),
                fields: fields("name", "record"),
            })))
        })
}

pub(super) fn role(
    index: usize,
    target: EntityReference,
    posture: RequiredOutputRolePosture,
) -> RequiredOutputRoleTarget {
    RequiredOutputRoleTarget {
        role: format!("member-{index}"),
        entity_name: "node".into(),
        kind: KindId(1),
        posture,
        target,
    }
}

pub(super) fn meaning(route: usize, source: EntityId) -> RequiredOutputPublicationMeaning {
    RequiredOutputPublicationMeaning {
        implementation: RequiredOutputProducingImplementation::from_checkpoint(
            [0x11; 16], [0x22; 32],
        )
        .unwrap(),
        family: format!("route-{route}"),
        source_scope: source,
        source_partition: worth_foundational::facade::CanonicalDigestId::new([0; 32]),
        source_epoch: worth_foundational::facade::CanonicalDigestId::new([1; 32]),
    }
}

pub(super) fn create_batch(
    source: EntityId,
    targets: &[CreatedEntityRef],
    route: usize,
    round: usize,
    edges: usize,
) -> (WorkerIntentBatch, Vec<CreatedRelationRef>) {
    let mut batch = creates(targets);
    let mut relations = Vec::new();
    assert!(edges <= targets.len() * targets.len());
    for i in 0..edges {
        let source_index = i % targets.len();
        let group = i / targets.len();
        let target = if group == 0 {
            EntityReference::Existing(source)
        } else {
            EntityReference::Created(targets[(source_index + group) % targets.len()].clone())
        };
        let edge = CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: ClientKey::raw(format!("route-{route}-round-{round}-edge-{i}")),
            source: EntityReference::Created(targets[i % targets.len()].clone()),
            target,
        };
        batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: edge.partition_id,
                kind_id: edge.kind_id,
                client_key: edge.client_key.clone(),
                source: edge.source.clone(),
                target: edge.target.clone(),
                fields: fields("label", "association"),
            },
        )));
        relations.push(edge);
    }
    (batch, relations)
}
