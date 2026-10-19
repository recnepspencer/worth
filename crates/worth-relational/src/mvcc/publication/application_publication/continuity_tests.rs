//! Real native partial/no-data completion and intervening-writer refusal.
use super::*;
use crate::identity::data::{EntityId, KindId, PartitionId};
use crate::tests::support::*;
use crate::transactions::data::*;
use worth_execution::ExecutionRequest;

fn publish(
    runtime: &crate::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    batch: WorkerIntentBatch,
    publication: RequiredOutputPublication,
) -> Result<CommitResult, TransactionCommitError> {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(batch, execution.allocation_policy())
        .unwrap();
    let proposal = transaction
        .validate(execution, runtime)?
        .bind_required_output_publication(installed, publication)?;
    let candidate = runtime.prepare_validated_proposal(execution, proposal)?;
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("native completion must perform")
    };
    runtime.settle_performed_publication(performed)
}

fn preserve(
    owner: &worth_proof::InstalledApplicationPublicationOwner,
    previous: RelationalRequiredOutputPublication,
    ids: [EntityId; 2],
    dependency: u8,
) -> RequiredOutputPublication {
    let admission = owner
        .admit_required_output_succession(
            owner.admit_required_output("native-preserver", [dependency; 32]),
            previous.authority(),
        )
        .unwrap();
    let meaning = previous
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap()
        .meaning
        .clone();
    RequiredOutputPublication::Preserve {
        admission,
        predecessor: previous,
        meaning,
        roles: ids
            .into_iter()
            .enumerate()
            .map(|(index, id)| RequiredOutputRoleTarget {
                role: format!("preserved-{index}"),
                entity_name: "native-unit-output".into(),
                kind: KindId(1),
                posture: RequiredOutputRolePosture::Preserve,
                target: EntityReference::Existing(id),
            })
            .collect(),
    }
}

#[test]
fn complete_partial_and_executed_no_data_publications_keep_unpatched_members_and_refuse_intervening_equal_write(
) {
    let request = crate::test_execution_policy::serial_request();
    let execution = &ExecutionRequest::serial(&request);
    let mut runtime = runtime_with_test_schema();
    let receipt = runtime
        .prepare_initial_schema_installation()
        .unwrap()
        .install(crate::schema::data::RelationalSchemaRegistry::new())
        .unwrap();
    let (owner, installed) = receipt.issue_application_publication_owner();
    let targets = ["left", "right"].map(|key| CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: KindId(1),
        client_key: crate::symbols::data::ClientKey::raw(key),
    });
    let mut batch = WorkerIntentBatch::new("actual-two-member-birth");
    for target in &targets {
        batch = batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: target.partition_id,
            kind_id: target.kind_id,
            client_key: target.client_key.clone(),
            fields: name_field_patch("original"),
        })));
    }
    let birth = publish(
        &runtime,
        execution,
        &installed,
        batch,
        native_birth_fixture(
            owner.admit_required_output("native-creator", [1; 32]),
            targets.to_vec(),
        ),
    )
    .unwrap();
    let ids = targets.map(|target| birth.created_entity(&target).unwrap());
    let original = birth.required_output_publication(&installed).unwrap();
    let born_at = birth.commit.commit_id;
    release_test_commit_snapshot(&runtime, &birth);
    let partial = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("actual-one-member-write").push(MutationIntent::Entity(
            EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                entity_id: ids[0],
                fields: name_field_patch("changed"),
            }),
        )),
        preserve(&owner, original, ids, 2),
    )
    .unwrap();
    let partial_id = partial.commit.commit_id;
    assert_eq!(partial.changed_records, [RecordRef::Entity(ids[0])]);
    let partial_origin = partial
        .publication()
        .envelope
        .required_output_origin
        .as_ref()
        .unwrap();
    assert_eq!(partial_origin.members.len(), 2);
    assert_eq!(partial_origin.birth, born_at);
    assert_ne!(
        partial_origin.members[0].after_write,
        partial_origin.members[1].after_write
    );
    let previous = partial.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &partial);
    let stale = previous.clone();
    let no_data = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("executed-no-data-completion"),
        preserve(&owner, previous, ids, 3),
    )
    .unwrap();
    assert!(no_data.changed_records.is_empty());
    assert_ne!(no_data.commit.commit_id, partial_id);
    let previous = no_data.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &no_data);
    let stale_denial = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("stale-predecessor-after-no-data"),
        preserve(&owner, stale, ids, 4),
    )
    .unwrap_err();
    assert!(format!("{stale_denial:?}")
        .contains("not the publication selected at the native before image"));
    let next = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("genuine-latest-predecessor-after-no-data"),
        preserve(&owner, previous, ids, 4),
    )
    .unwrap();
    assert!(next.changed_records.is_empty());
    let no_data_id = next.commit.commit_id;
    let previous = next.required_output_publication(&installed).unwrap();
    release_test_commit_snapshot(&runtime, &next);
    let basis = test_owner_main_basis(&runtime).unwrap();
    let cold = runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[4; 32],
            &crate::mvcc::native_meaning_fixture("native-creator"),
            &crate::mvcc::native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                crate::mvcc::RequiredOutputRolePosture::Preserve,
            ),
        )
        .unwrap();
    assert_eq!(
        cold.envelope.commit.commit_id, no_data_id,
        "membership wins over two different latest member writes"
    );
    for subset in [&ids[..1], &ids[1..]] {
        assert!(runtime
            .readmit_required_output_publication(
                execution,
                &installed,
                &basis,
                "native-preserver",
                &[4; 32],
                &crate::mvcc::native_meaning_fixture("native-creator"),
                &crate::mvcc::native_roles_fixture(
                    &subset.iter().map(|id| (*id, KindId(1))).collect::<Vec<_>>(),
                    crate::mvcc::RequiredOutputRolePosture::Preserve
                )
            )
            .is_err());
    }
    let loci = [crate::runtime::CodecFieldLocus {
        kind_id: KindId(1),
        aspect_key: aspect_key("name"),
        field_key: field_key("name"),
        binding: worth_foundational::facade::AspectBinding::EntityField {
            field: field_key("name"),
        },
    }];
    let all =
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &loci)
            .unwrap();
    assert_eq!(all.entities, ids);
    let none =
        crate::history::data::derive_codec_retirement_closure(execution, &runtime, &basis, &[])
            .unwrap();
    assert!(none.entities.is_empty());
    let mut ordinary = test_owner_begin_transaction_for_main(&runtime);
    ordinary
        .push_batch(
            WorkerIntentBatch::new("ordinary-equal-overwrite-between-publications").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: ids[0],
                        fields: name_field_patch("changed"),
                    },
                )),
            ),
            execution.allocation_policy(),
        )
        .unwrap();
    let overwrite = ordinary.commit(execution, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &overwrite);
    let before = runtime
        .durability_authority()
        .native_checkpoint(execution.allocation_policy())
        .unwrap();
    let denied = publish(
        &runtime,
        execution,
        &installed,
        WorkerIntentBatch::new("attempted-no-data-preserve-after-overwrite"),
        preserve(&owner, previous, ids, 5),
    )
    .unwrap_err();
    assert!(format!("{denied:?}").contains("intervening native write revoked producer ownership"));
    assert_eq!(
        runtime
            .durability_authority()
            .native_checkpoint(execution.allocation_policy())
            .unwrap(),
        before
    );
    let basis = test_owner_main_basis(&runtime).unwrap();
    assert!(runtime
        .readmit_required_output_publication(
            execution,
            &installed,
            &basis,
            "native-preserver",
            &[4; 32],
            &crate::mvcc::native_meaning_fixture("native-creator"),
            &crate::mvcc::native_roles_fixture(
                &ids.map(|id| (id, KindId(1))),
                crate::mvcc::RequiredOutputRolePosture::Preserve
            )
        )
        .is_err());
}

#[path = "continuity_tests/dynamic.rs"]
mod dynamic;

#[path = "continuity_tests/partial_fields.rs"]
mod partial_fields;

#[path = "continuity_tests/retirement.rs"]
mod retirement;

#[path = "continuity_tests/routes.rs"]
mod routes;

#[path = "continuity_tests/fieldless.rs"]
mod fieldless;
