//! Public warm and cold retirement admission for selected Native routes.
mod authored_refusal;
mod ordinary;

use super::*;
use crate::index_retirement::{create_entity_outcome, runtime_with_test_schema};
use worth_relational::facade::{
    history::BranchId,
    identity::RelationId,
    schema::RelationalSchemaRegistry,
    transactions::{
        CreateIntent, CreatedRelationRef, DeleteEntityIntent, DeleteRelationIntent,
        EntityMutationIntent, EntitySpec, MutationIntent, RelationMutationIntent, RelationSpec,
    },
};
use worth_testkit_relational::{
    aspect_storage::{
        aspect_key, field_key,
        publications::{changed_entities, release_test_commit_snapshot, string_aspect_field_patch},
    },
    shared_relational_certification_test_execution_policy::serial_request,
};

fn name_field_patch(value: &str) -> worth_relational::facade::transactions::AspectFieldPatch {
    string_aspect_field_patch([(aspect_key("name"), field_key("name"), value)])
}

fn creates(keys: [&str; 2]) -> (WorkerIntentBatch, [CreatedEntityRef; 2]) {
    let targets = keys.map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: key.into(),
    });
    let mut batch = WorkerIntentBatch::new("real-new-complete-membership");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: name_field_patch("actual payload"),
        })));
    }
    (batch, targets)
}

fn completion(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    roles: Vec<RequiredOutputRoleTarget>,
    dependency: u8,
) -> RequiredOutputPublication {
    let meaning = previous.meaning().clone();
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("retirement-preserver", [dependency; 32]),
            previous.authority(),
        )
        .unwrap();
    RequiredOutputPublication::Preserve {
        admission,
        predecessor: previous,
        meaning,
        roles,
    }
}

fn birth_with_edge(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
) -> (CommitResult, [EntityId; 2], RelationId) {
    let (batch, targets) = creates(["authored-left", "authored-right"]);
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: "authored-owned-edge".into(),
        source: EntityReference::Created(targets[0].clone()),
        target: EntityReference::Created(targets[1].clone()),
    };
    let batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: edge.partition_id,
            kind_id: edge.kind_id,
            client_key: edge.client_key.clone(),
            source: edge.source.clone(),
            target: edge.target.clone(),
            fields: string_aspect_field_patch([(
                aspect_key("label"),
                field_key("label"),
                "owned edge",
            )]),
        },
    )));
    let born = publish(
        runtime,
        execution,
        installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("authored-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| born.created_entity(&target).unwrap());
    let relation = born.created_relation(&edge).unwrap();
    #[derive(serde::Deserialize)]
    struct CanonicalMembership {
        outputs: Vec<EntityId>,
        relations: Vec<RelationMembership>,
    }
    #[derive(serde::Deserialize)]
    struct RelationMembership {
        id: RelationId,
        source: EntityId,
        target: EntityId,
    }
    #[derive(serde::Deserialize)]
    struct MembershipCarrier {
        required_output_origin: CanonicalMembership,
    }
    let payload = rmp_serde::to_vec_named(&*born.publication().envelope).unwrap();
    let membership = rmp_serde::from_slice::<MembershipCarrier>(&payload)
        .unwrap()
        .required_output_origin;
    assert_eq!(membership.outputs, ids);
    assert_eq!(membership.relations.len(), 1);
    assert_eq!(
        (
            membership.relations[0].id,
            membership.relations[0].source,
            membership.relations[0].target
        ),
        (relation, ids[0], ids[1])
    );
    (born, ids, relation)
}

fn retire_roles(ids: [EntityId; 2]) -> Vec<RequiredOutputRoleTarget> {
    native_roles_fixture(
        &ids.map(|id| (id, KindId(1))),
        RequiredOutputRolePosture::Retire,
    )
}

fn authored(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    predecessor: RelationalRequiredOutputPublication,
    roles: Vec<RequiredOutputRoleTarget>,
) -> RequiredOutputPublication {
    let admission = owner
        .admit_authored_output_retirement(predecessor.authority())
        .unwrap();
    RequiredOutputPublication::AuthoredRetirement {
        admission,
        predecessor,
        roles,
    }
}

fn deletes(ids: [EntityId; 2], relation: RelationId) -> WorkerIntentBatch {
    let mut batch = WorkerIntentBatch::new("ordinary-authored-complete-retirement").push(
        MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
            relation_id: relation,
        })),
    );
    for id in ids {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: id },
        )));
    }
    batch
}
