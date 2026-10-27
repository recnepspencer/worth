//! Real producer ownership must survive exact subset materialization maintenance.
use super::*;
use crate::branch::{RelationalEntityMaterialization, RelationalRelationMaterialization};

#[path = "materialization/incident_owner.rs"]
mod incident_owner;

#[test]
fn exact_subset_rematerialization_keeps_cold_producer_custody_and_ordinary_succession() {
    run(true, 0, false);
}

#[test]
fn changed_or_unattested_rematerialization_does_not_acquire_producer_custody() {
    run(true, 1, false);
    run(true, 2, false);
    run(false, 0, false);
}

#[test]
fn fieldless_owned_relation_birth_survives_materialization_and_cold_succession() {
    run(true, 0, true);
}

fn run(owned: bool, changed: u8, fieldless: bool) {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let make_runtime = || {
        if fieldless {
            AspectSchemaFixture::default().build_runtime()
        } else {
            runtime_with_test_schema()
        }
    };
    let mut runtime = make_runtime();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["suspended-member", "retained-member"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let relation_key = crate::symbols::data::ClientKey::raw("owned-materialization-edge");
    let mut batch = WorkerIntentBatch::new("actual-materialization-producer");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: if fieldless {
                AspectFieldPatch::default()
            } else {
                name_field_patch("original")
            },
        })));
    }
    let relation_fields = if fieldless {
        AspectFieldPatch::default()
    } else {
        relation_label_field_patch("original relation")
    };
    batch = batch.push(MutationIntent::Create(CreateIntent::Relation(
        RelationSpec {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: relation_key.clone(),
            source: EntityReference::Created(targets[0].clone()),
            target: EntityReference::Created(targets[1].clone()),
            fields: relation_fields.clone(),
        },
    )));
    let birth = if owned {
        publish(
            &runtime,
            execution,
            &installed,
            batch,
            native_birth_fixture(
                owner.admit_required_output("native-creator", [1; 32]),
                targets.to_vec(),
            ),
        )
        .unwrap()
    } else {
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        transaction.commit(execution, &runtime).unwrap()
    };
    let relation = birth
        .created_relation(&CreatedRelationRef {
            partition_id: PartitionId::main(),
            kind_id: KindId(2),
            client_key: relation_key,
            source: EntityReference::Created(targets[0].clone()),
            target: EntityReference::Created(targets[1].clone()),
        })
        .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let original_commit = birth.commit.commit_id;
    if fieldless {
        let patch = birth
            .publication()
            .envelope
            .patch
            .authoritative_record_patches
            .iter()
            .find(|patch| patch.target == RecordRef::Relation(relation))
            .unwrap();
        assert_eq!(
            patch.authoritative_patch.full_grammar_operation_count(),
            0,
            "native relation endpoints need no exposed endpoint aspects"
        );
    }
    drop(birth.required_output_publication(&installed));
    release_test_commit_snapshot(&runtime, &birth);
    let services = runtime.owner_component_services();
    for _ in 0..if changed == 0 && owned && !fieldless {
        2
    } else {
        1
    } {
        let basis = test_owner_main_basis(&runtime).unwrap();
        let suspension = services
            .materialization_port()
            .prepare_generated_materialization_suspension(execution, &basis, &ids[..1])
            .unwrap();
        let (candidate, completion) = suspension.into_parts();
        let suspended = completion
            .complete(
                runtime
                    .publish_prepared_candidate(execution, candidate)
                    .unwrap(),
            )
            .unwrap();
        assert_eq!(
            suspended.custody.records().len(),
            2,
            "member and its real incident relation"
        );
        release_test_commit_snapshot(&runtime, &suspended.commit);
        let basis = test_owner_main_basis(&runtime).unwrap();
        let reconstruction = services
            .materialization_port()
            .prepare_generated_rematerialization(
                execution,
                &basis,
                suspended.custody,
                vec![RelationalEntityMaterialization {
                    entity_id: ids[0],
                    kind_id: KindId(1),
                    fields: if fieldless {
                        AspectFieldPatch::default()
                    } else {
                        name_field_patch(if changed == 1 {
                            "different"
                        } else {
                            "original"
                        })
                    },
                }],
                vec![RelationalRelationMaterialization {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: ids[0],
                    target: ids[1],
                    fields: if changed == 2 {
                        relation_label_field_patch("different relation")
                    } else {
                        relation_fields.clone()
                    },
                }],
            )
            .unwrap();
        let (candidate, completion, _) = reconstruction.into_parts();
        let restored = completion
            .complete(
                runtime
                    .publish_prepared_candidate(execution, candidate)
                    .unwrap(),
            )
            .unwrap();
        release_test_commit_snapshot(&runtime, &restored);
    }
    if fieldless {
        runtime.retention().run_pass(execution).unwrap();
        let edition = runtime.acquire_partition_edition();
        let partition = edition.partition(relation.partition_id).unwrap();
        assert!(
            partition
                .relation_arena
                .metadata_history_at(relation.slot_index())
                .unwrap()
                .iter()
                .all(|metadata| metadata.effective_at != birth.commit.version_id),
            "the native birth metadata was actually retired before cold readmission"
        );
    }
    let saved = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    drop((services, birth, owner, installed));
    drop(runtime);
    let mut recovered = make_runtime();
    let receipt = recovered
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    recovered
        .durability_recovery()
        .restore_native_checkpoint(execution, &saved)
        .unwrap();
    let receipt = recovered
        .readmit_recovered_initial_schema_installation(receipt)
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let runtime = recovered;
    let basis = test_owner_main_basis(&runtime).unwrap();
    let cold = runtime.readmit_required_output_publication(
        execution,
        &installed,
        &basis,
        "native-creator",
        &[1; 32],
        &crate::mvcc::native_meaning_fixture("native-creator"),
        &crate::mvcc::native_roles_fixture(
            &ids.map(|id| (id, KindId(1))),
            RequiredOutputRolePosture::Create,
        ),
    );
    if changed != 0 || !owned {
        assert!(
            cold.is_err(),
            "generic restoration cannot establish producer ownership"
        );
        return;
    }
    let cold =
        cold.expect("exact owned maintenance keeps the actual complete producer association");
    assert_eq!(
        crate::history::data::readmit_required_output_route(
            execution,
            &runtime,
            &basis,
            &crate::mvcc::native_meaning_fixture("native-creator")
        )
        .unwrap(),
        Some(crate::history::data::RequiredOutputRoutePosition::Published(original_commit))
    );
    let origin = cold.envelope.required_output_origin.as_ref().unwrap();
    assert_eq!(origin.birth, original_commit);
    assert_eq!(origin.birth_producer, "native-creator");
    assert_eq!(origin.producer, "native-creator");
    assert_eq!(
        origin.meaning,
        crate::mvcc::native_meaning_fixture("native-creator")
    );
    let preserved = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("ordinary-preserve-after-materialization"),
        preserve(&owner, cold, ids, 2),
    )
    .unwrap();
    release_test_commit_snapshot(&runtime, &preserved);
    runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &test_owner_main_basis(&runtime).unwrap(),
            "native-preserver",
            &[2; 32],
            &crate::mvcc::native_meaning_fixture("native-creator"),
            &crate::mvcc::native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                RequiredOutputRolePosture::Preserve,
            ),
        )
        .expect("ordinary succession retains complete entity and owned relation ancestry");
}
