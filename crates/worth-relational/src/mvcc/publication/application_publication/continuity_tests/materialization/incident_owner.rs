//! Relation custody follows its producer even when only its other endpoint suspends.
use super::*;

#[test]
fn retained_producer_keeps_owned_incident_relation_when_other_group_endpoint_is_suspended() {
    run(false);
}

#[test]
fn current_group_survives_owned_birth_endpoint_replacement_retirement_and_omission() {
    run(true);
}

fn run(replace_endpoint: bool) {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let target = |key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    };
    let b = target("suspended-group-b");
    let first = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("group-b").push(MutationIntent::Create(CreateIntent::Entity(
            EntitySpec {
                partition_id: b.partition_id,
                kind_id: b.kind_id,
                client_key: b.client_key.clone(),
                fields: name_field_patch("group b"),
            },
        ))),
        native_birth_fixture(
            owner.admit_required_output("group-b", [1; 32]),
            vec![b.clone()],
        ),
    )
    .unwrap();
    let b = first.created_entity(&b).unwrap();
    release_test_commit_snapshot(&runtime, &first);
    let a = target("retained-group-a");
    let edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("a-owned-edge-to-b"),
        source: EntityReference::Created(a.clone()),
        target: EntityReference::Existing(b),
    };
    let second = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("group-a")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: a.partition_id,
                kind_id: a.kind_id,
                client_key: a.client_key.clone(),
                fields: name_field_patch("group a"),
            })))
            .push(MutationIntent::Create(CreateIntent::Relation(
                RelationSpec {
                    partition_id: edge.partition_id,
                    kind_id: edge.kind_id,
                    client_key: edge.client_key.clone(),
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    fields: relation_label_field_patch("a owns this relation"),
                },
            ))),
        native_birth_fixture(
            owner.admit_required_output("group-a", [2; 32]),
            vec![a.clone()],
        ),
    )
    .unwrap();
    let a = second.created_entity(&a).unwrap();
    let relation = second.created_relation(&edge).unwrap();
    let a_birth = second.commit.commit_id;
    let (a, a_producer, a_dependency, a_posture) = if replace_endpoint {
        (
            replace_birth_endpoint(
                &runtime,
                execution,
                &installed,
                &owner,
                second.required_output_publication(&installed).unwrap(),
                a,
                b,
                relation,
            ),
            "group-a-current",
            [5; 32],
            RequiredOutputRolePosture::Preserve,
        )
    } else {
        (a, "group-a", [2; 32], RequiredOutputRolePosture::Create)
    };
    drop((
        first.required_output_publication(&installed),
        second.required_output_publication(&installed),
    ));
    release_test_commit_snapshot(&runtime, &second);
    let services = runtime.owner_component_services();
    let basis = test_owner_main_basis(&runtime).unwrap();
    let (candidate, completion) = services
        .materialization_port()
        .prepare_generated_materialization_suspension(execution, &basis, &[b])
        .unwrap()
        .into_parts();
    let suspended = completion
        .complete(
            runtime
                .publish_prepared_candidate(execution, candidate)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(suspended.custody.records().len(), 2);
    release_test_commit_snapshot(&runtime, &suspended.commit);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let (candidate, completion, _) = services
        .materialization_port()
        .prepare_generated_rematerialization(
            execution,
            &basis,
            suspended.custody,
            vec![RelationalEntityMaterialization {
                entity_id: b,
                kind_id: KindId(1),
                fields: name_field_patch("group b"),
            }],
            vec![RelationalRelationMaterialization {
                relation_id: relation,
                kind_id: KindId(2),
                source: a,
                target: b,
                fields: relation_label_field_patch("a owns this relation"),
            }],
        )
        .unwrap()
        .into_parts();
    let restored = completion
        .complete(
            runtime
                .publish_prepared_candidate(execution, candidate)
                .unwrap(),
        )
        .unwrap();
    release_test_commit_snapshot(&runtime, &restored);
    let basis = test_owner_main_basis(&runtime).unwrap();
    for (producer, family, dependency, id, birth, posture) in [
        (
            "group-b",
            "group-b",
            [1; 32],
            b,
            first.commit.commit_id,
            RequiredOutputRolePosture::Create,
        ),
        (a_producer, "group-a", a_dependency, a, a_birth, a_posture),
    ] {
        let cold = runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &basis,
                producer,
                &dependency,
                &crate::mvcc::native_meaning_fixture(family),
                &crate::mvcc::native_roles_fixture(&[(id, KindId(1))], posture),
            )
            .expect(
                "both complete groups retain custody, including the retained relation producer",
            );
        assert_eq!(
            cold.envelope.required_output_origin.as_ref().unwrap().birth,
            birth
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn replace_birth_endpoint(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    original: RelationalRequiredOutputPublication,
    old: EntityId,
    other: EntityId,
    relation: crate::identity::data::RelationId,
) -> EntityId {
    let replacement = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("replacement-a"),
    };
    let mut roles =
        crate::mvcc::native_roles_fixture(&[(old, KindId(1))], RequiredOutputRolePosture::Preserve);
    roles.push(RequiredOutputRoleTarget {
        role: "joined-a".into(),
        entity_name: "native-unit-output".into(),
        kind: KindId(1),
        posture: RequiredOutputRolePosture::Create,
        target: EntityReference::Created(replacement.clone()),
    });
    let moved = publish(
        runtime,
        execution,
        installed,
        WorkerIntentBatch::new("replace-and-rewire-a")
            .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
                partition_id: replacement.partition_id,
                kind_id: replacement.kind_id,
                client_key: replacement.client_key.clone(),
                fields: name_field_patch("replacement group a"),
            })))
            .push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Created(replacement.clone()),
                    target: EntityReference::Existing(other),
                }),
            )),
        complete(owner, original, roles, 3),
    )
    .unwrap();
    let joined = moved.created_entity(&replacement).unwrap();
    release_test_commit_snapshot(runtime, &moved);
    let mut roles =
        crate::mvcc::native_roles_fixture(&[(old, KindId(1))], RequiredOutputRolePosture::Retire);
    roles[0].role = "retired-a".into();
    roles.extend(crate::mvcc::native_roles_fixture(
        &[(joined, KindId(1))],
        RequiredOutputRolePosture::Preserve,
    ));
    let retired = publish(
        runtime,
        execution,
        installed,
        WorkerIntentBatch::new("retire-rewired-away-a").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: old }),
        )),
        complete(
            owner,
            moved.required_output_publication(installed).unwrap(),
            roles,
            4,
        ),
    )
    .unwrap();
    release_test_commit_snapshot(runtime, &retired);
    let current = publish(
        runtime,
        execution,
        installed,
        WorkerIntentBatch::new("omit-old-retired-role"),
        complete(
            owner,
            retired.required_output_publication(installed).unwrap(),
            crate::mvcc::native_roles_fixture(
                &[(joined, KindId(1))],
                RequiredOutputRolePosture::Preserve,
            ),
            5,
        ),
    )
    .unwrap();
    release_test_commit_snapshot(runtime, &current);
    let cold = runtime
        .readmit_required_output_publication(
            execution,
            installed,
            &test_owner_main_basis(runtime).unwrap(),
            "group-a-current",
            &[5; 32],
            &crate::mvcc::native_meaning_fixture("group-a"),
            &crate::mvcc::native_roles_fixture(
                &[(joined, KindId(1))],
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .expect("the later complete publication is valid before materialization");
    assert_eq!(cold.envelope.commit.commit_id, current.commit.commit_id);
    joined
}

fn complete(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    roles: Vec<RequiredOutputRoleTarget>,
    dependency: u8,
) -> RequiredOutputPublication {
    let meaning = previous
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("group-a-current", [dependency; 32]),
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
