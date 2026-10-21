//! One real succession replaces a dynamic member and retains its complete roster.
use super::*;

#[test]
fn dynamic_create_preserve_retire_tracks_member_birth_and_requires_prior_live_coverage() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["root", "old-leaf"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("dynamic-family-birth");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: name_field_patch("original"),
        })));
    }
    let old_edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("old-edge"),
        source: EntityReference::Created(targets[0].clone()),
        target: EntityReference::Created(targets[1].clone()),
    };
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: old_edge.partition_id,
            kind_id: old_edge.kind_id,
            client_key: old_edge.client_key.clone(),
            source: old_edge.source.clone(),
            target: old_edge.target.clone(),
            fields: string_aspect_field_patch([(
                aspect_key("label"),
                field_key("label"),
                "actual old edge",
            )]),
        },
    )));
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("dynamic-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let retired_edge = birth.created_relation(&old_edge).unwrap();
    let original = birth.required_output_publication(&installed).unwrap();
    let family_birth = birth.commit.commit_id;
    release_test_commit_snapshot(&runtime, &birth);
    let joined = CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw("new-leaf"),
    };
    let joined_edge = CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(2),
        client_key: crate::symbols::data::ClientKey::raw("new-edge"),
        source: EntityReference::Existing(ids[0]),
        target: EntityReference::Created(joined.clone()),
    };
    let batch = WorkerIntentBatch::new("real-dynamic-member-replacement")
        .push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: ids[1] },
        )))
        .push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: joined.partition_id,
            kind_id: joined.kind_id,
            client_key: joined.client_key.clone(),
            fields: name_field_patch("new payload"),
        })))
        .push(MutationIntent::Create(CreateIntent::Relation(
            RelationSpec {
                partition_id: joined_edge.partition_id,
                kind_id: joined_edge.kind_id,
                client_key: joined_edge.client_key.clone(),
                source: joined_edge.source.clone(),
                target: joined_edge.target.clone(),
                fields: string_aspect_field_patch([(
                    aspect_key("label"),
                    field_key("label"),
                    "actual new edge",
                )]),
            },
        )));
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("dynamic-preserver", [2; 32]),
            original.authority(),
        )
        .unwrap();
    let meaning = original
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    let roles = [
        (
            "root",
            RequiredOutputRolePosture::Preserve,
            EntityReference::Existing(ids[0]),
        ),
        (
            "retired-leaf",
            RequiredOutputRolePosture::Retire,
            EntityReference::Existing(ids[1]),
        ),
        (
            "joined-leaf",
            RequiredOutputRolePosture::Create,
            EntityReference::Created(joined.clone()),
        ),
    ]
    .map(|(role, posture, target)| RequiredOutputRoleTarget {
        role: role.into(),
        entity_name: "native-unit-output".into(),
        kind: KindId(1),
        posture,
        target,
    });
    let changed = publish(
        &runtime,
        execution,
        &installed,
        batch,
        RequiredOutputPublication::Preserve {
            admission,
            predecessor: original,
            meaning,
            roles: roles.to_vec(),
        },
    )
    .unwrap();
    let joined_id = changed.created_entity(&joined).unwrap();
    let joined_edge_id = changed.created_relation(&joined_edge).unwrap();
    let successor = changed.required_output_publication(&installed).unwrap();
    let origin = successor.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.birth, family_birth);
    assert!(!origin.relations.iter().any(|edge| edge.id == retired_edge));
    assert_eq!(origin.relations.len(), 1);
    assert_eq!(origin.relations[0].id, joined_edge_id);
    assert_eq!(origin.relations[0].birth, changed.commit.commit_id);
    let member = origin
        .members
        .iter()
        .find(|member| member.id == joined_id)
        .unwrap();
    assert_eq!(member.birth, changed.commit.commit_id);
    assert_eq!(member.birth_producer, "dynamic-preserver");
    let mut cold_roles = roles.to_vec();
    cold_roles[2].target = EntityReference::Existing(joined_id);
    release_test_commit_snapshot(&runtime, &changed);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: worth_foundational::facade::AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    let closed =
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &loci)
            .unwrap();
    assert_eq!(closed.entities, [ids[0], joined_id]);
    assert_eq!(closed.relations, [joined_edge_id]);
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "dynamic-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &cold_roles,
        )
        .unwrap();
    // Cross actual native encoding/readmission, then publish from the missing
    // runtime position image using the exact cold-admitted before-basis custody.
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let mut reopened = runtime_with_test_schema();
    let initial_receipt = reopened
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    reopened
        .durability_recovery()
        .restore_native_checkpoint(execution, &checkpoint)
        .unwrap();
    let fresh_receipt = reopened
        .readmit_recovered_initial_schema_installation(initial_receipt)
        .unwrap();
    let (fresh_owner, fresh_installed) = fresh_receipt.issue_application_publication_owner();
    let fresh_basis = test_owner_main_basis(&reopened).unwrap();
    let cold = reopened
        .readmit_required_output_publication(
            execution,
            &fresh_installed,
            &fresh_basis,
            "dynamic-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &cold_roles,
        )
        .unwrap();
    let renewed = publish(
        &reopened,
        execution,
        &fresh_installed,
        WorkerIntentBatch::new("genuine-executed-no-data-after-native-restore"),
        preserve(&fresh_owner, cold, [ids[0], joined_id], 3),
    )
    .unwrap();
    assert!(renewed.changed_records.is_empty());
    release_test_commit_snapshot(&reopened, &renewed);
    let renewed_basis = test_owner_main_basis(&reopened).unwrap();
    reopened
        .readmit_required_output_publication(
            execution,
            &fresh_installed,
            &renewed_basis,
            "native-preserver",
            &[3; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &crate::mvcc::native_roles_fixture(
                &[(ids[0], KindId(1)), (joined_id, KindId(1))],
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
    // A dead member is still part of this performed correspondence. Omitting
    // it cannot turn a partial description into a complete native association.
    assert!(runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "dynamic-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &[cold_roles[0].clone(), cold_roles[2].clone()]
        )
        .is_err());
    let roles_with_wrong_posture = [RequiredOutputRoleTarget {
        posture: RequiredOutputRolePosture::Preserve,
        ..cold_roles[1].clone()
    }];
    assert!(runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "dynamic-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &roles_with_wrong_posture
        )
        .is_err());
    let mut incomplete = preserve(&owner, successor.clone(), [ids[0], joined_id], 3);
    if let RequiredOutputPublication::Preserve { roles, .. } = &mut incomplete {
        roles.pop();
    }
    assert!(publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("omitted-live-member"),
        incomplete
    )
    .is_err());
    let completed = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("next-dynamic-no-data"),
        preserve(&owner, successor, [ids[0], joined_id], 3),
    )
    .unwrap();
    assert!(completed.changed_records.is_empty());
    release_test_commit_snapshot(&runtime, &completed);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let closed =
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &loci)
            .unwrap();
    assert_eq!(closed.relations, [joined_edge_id]);
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[3; 32],
            &crate::mvcc::native_meaning_fixture("dynamic-creator"),
            &crate::mvcc::native_roles_fixture(
                &[(ids[0], KindId(1)), (joined_id, KindId(1))],
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
}
