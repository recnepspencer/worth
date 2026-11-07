use super::*;
use crate::facade::indexes::{DerivedIndexEntries, DerivedIndexMaintenanceBudget};

#[test]
fn unrelated_field_indexes_do_not_reread_changed_records() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_ids = ["name", "unrelated-a", "unrelated-b", "unrelated-c"]
        .into_iter()
        .map(|name| {
            runtime
                .index_authority()
                .register(DerivedIndexDefinition {
                    index_id: DerivedIndexId(0),
                    name: format!("maintenance.{name}"),
                    kind: DerivedIndexKind::EntityField {
                        field_locator: aspect_field_locator(aspect_key(name), field_key(name)),
                    },
                    branch_scoped: true,
                })
                .expect("index fixture has identity capacity")
                .index_id
        })
        .collect::<Vec<_>>();
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: index_ids.clone(),
    };
    let initial = runtime
        .index_authority()
        .build_for_commit(execution, request(first.commit.commit_id));
    assert!(initial.failed_indexes.is_empty());
    let candidate =
        candidate_index_publication::candidate_for_update(execution, &runtime, entity, "after");
    let native_work = candidate.prepared_index_maintenance_work().unwrap();
    let second = candidate_index_publication::publish_native(execution, &runtime, candidate);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(100_000),
                maximum_cold_record_slots: Some(0),
                maximum_derived_rows: Some(0),
            },
            execution,
        )
        .unwrap();
    assert_eq!(refreshed.work.patch_records, 1);
    assert_eq!(native_work.record_reads, 2);
    assert_eq!(native_work.entry_edits, 2);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn relation_field_deletion_removes_the_old_entry_with_one_read_per_basis() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let first = create_relation_outcome(execution, &runtime, source, target, "link");
    let relation = changed_relations(&first)[0];
    let index = runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "maintenance.deleted-relation".into(),
            kind: DerivedIndexKind::RelationField {
                field_locator: aspect_field_locator(aspect_key("label"), field_key("label")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity");
    let request = |commit_id| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids: vec![index.index_id],
    };
    let initial = runtime
        .index_authority()
        .build_for_commit(execution, request(first.commit.commit_id));
    assert!(initial.failed_indexes.is_empty());
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("delete-indexed-relation").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
            execution.allocation_policy(),
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    let native_work = candidate.prepared_index_maintenance_work().unwrap();
    let second = candidate_index_publication::publish_native(execution, &runtime, candidate);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id),
            &basis,
            Some(&first.snapshot),
            budget(),
            execution,
        )
        .unwrap();
    assert_eq!(native_work.record_reads, 2);
    assert_eq!(native_work.entry_edits, 1);
    let DerivedIndexEntries::RelationField(entries) = &refreshed.generations[0].entries else {
        panic!("relation field index")
    };
    assert!(entries.is_empty());
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn mixed_seeded_and_missing_fields_refuse_until_explicit_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let register = |name: &str, field: &str| {
        runtime
            .index_authority()
            .register(DerivedIndexDefinition {
                index_id: DerivedIndexId(0),
                name: format!("maintenance.{name}"),
                kind: DerivedIndexKind::EntityField {
                    field_locator: aspect_field_locator(aspect_key(field), field_key(field)),
                },
                branch_scoped: true,
            })
            .expect("index fixture has identity capacity")
    };
    let name = register("name", "name");
    let request = |commit_id, index_ids| DerivedIndexBuildRequest {
        source_commit_id: commit_id,
        branch_id: BranchId("main".into()),
        index_ids,
    };
    let initial = runtime.index_authority().build_for_commit(
        execution,
        request(first.commit.commit_id, vec![name.index_id]),
    );
    assert!(initial.failed_indexes.is_empty());
    let cold = register("name-cold", "name");
    let transaction =
        candidate_index_publication::transaction_for_update(&runtime, entity, "after");
    let refused = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap_err();
    assert!(
        matches!(refused, crate::transactions::data::TransactionCommitError::Publication { ref error, .. } if error.detail.contains("ColdReconstructionRequired"))
    );
    assert_eq!(runtime.history().immutable_commit_count(), 1);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refused = runtime
        .index_authority()
        .refresh_for_basis(
            request(first.commit.commit_id, vec![name.index_id, cold.index_id]),
            &basis,
            None,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
            execution,
        )
        .unwrap_err();
    assert_eq!(
        refused.kind,
        crate::indexes::data::DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    assert_eq!(refused.work.cold_record_slots, 0);
    runtime
        .index_authority()
        .reconstruct_for_basis(
            execution,
            request(first.commit.commit_id, vec![cold.index_id]),
            &basis,
            DerivedIndexMaintenanceBudget {
                maximum_work_units: None,
                maximum_cold_record_slots: None,
                maximum_derived_rows: None,
            },
        )
        .unwrap();
    let candidate =
        candidate_index_publication::candidate_for_update(execution, &runtime, entity, "after");
    let native_work = candidate.prepared_index_maintenance_work().unwrap();
    let second = candidate_index_publication::publish_native(execution, &runtime, candidate);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refreshed = runtime
        .index_authority()
        .refresh_for_basis(
            request(second.commit.commit_id, vec![name.index_id, cold.index_id]),
            &basis,
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: Some(100_000),
                maximum_cold_record_slots: Some(1_000),
                maximum_derived_rows: Some(0),
            },
            execution,
        )
        .unwrap();
    assert_eq!(refreshed.work.cold_record_slots, 0);
    assert_eq!(native_work.record_reads, 2);
    assert_eq!(native_work.entry_edits, 4);
    let name_generation = refreshed
        .generations
        .iter()
        .find(|generation| generation.index_id == name.index_id)
        .unwrap();
    let DerivedIndexEntries::EntityField(name_entries) = &name_generation.entries else {
        panic!("name field index")
    };
    assert_eq!(name_entries.len(), 1);
    let cold_generation = refreshed
        .generations
        .iter()
        .find(|generation| generation.index_id == cold.index_id)
        .unwrap();
    let DerivedIndexEntries::EntityField(cold_entries) = &cold_generation.entries else {
        panic!("cold field index")
    };
    assert_eq!(cold_entries, name_entries);
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(100_000),
        maximum_cold_record_slots: Some(0),
        maximum_derived_rows: Some(0),
    }
}
