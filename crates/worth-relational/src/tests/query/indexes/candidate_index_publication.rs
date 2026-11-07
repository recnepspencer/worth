#[path = "candidate_index_publication/incremental_accounting.rs"]
mod incremental_accounting;
use super::*;
use crate::facade::indexes::{DerivedIndexEntries, DerivedIndexMaintenanceDenialKind};
use crate::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use crate::storage::data::AuthoritativeFieldComparisonKey;

fn index(runtime: &RelationalRuntime) -> DerivedIndexId {
    runtime
        .index_authority()
        .register(DerivedIndexDefinition {
            index_id: DerivedIndexId(0),
            name: "candidate.name".into(),
            kind: DerivedIndexKind::EntityField {
                field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
            },
            branch_scoped: true,
        })
        .expect("index fixture has identity capacity")
        .index_id
}

pub(super) fn candidate_for_update(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) -> crate::mvcc::PreparedRelationalCommitCandidate {
    runtime
        .prepare_branch_transaction(execution, transaction_for_update(runtime, entity_id, name))
        .unwrap()
}

pub(super) fn transaction_for_update(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) -> crate::mvcc::BranchBoundRelationalTransaction {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("candidate-index-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: name_field_patch(name),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    transaction
}

#[test]
fn expired_native_candidate_defers_publication_without_exposing_indexes() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: None,
            max_published_snapshot_handles: Some(8),
            max_active_snapshot_handles: Some(8),
            max_transaction_savepoints: 8,
            max_prepared_candidates: Some(1),
            candidate_max_lifetime_millis: 0,
        })
        .build()
        .expect("valid native runtime configuration");
    let _index_id = index(&runtime);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("expired-index-candidate"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();

    assert!(candidate.prepared_index_maintenance_work().is_some());
    let outcome = runtime
        .publication_port()
        .compare_and_publish(execution, candidate);
    assert!(matches!(
        outcome,
        crate::mvcc::RelationalPublicationOutcome::Deferred(_)
    ));
    assert_eq!(runtime.index_access().generations_snapshot().len(), 0);
    assert_eq!(runtime.history().immutable_commit_count(), 0);
}

#[test]
fn native_empty_predecessor_and_seeded_updates_match_cold_at_every_step() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime_with_index_field_aspects();
    let index_id = index(&runtime);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch_create("initial"),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let first = publish_native(
        execution,
        &runtime,
        runtime
            .prepare_branch_transaction(execution, transaction)
            .unwrap(),
    );
    let entity = changed_entities(&first)[0];
    let mut court = worth_testkit_relational::maintenance::MaintainedCourt::new(
        "native_indexes",
        Arc::new(Vec::new()),
    );
    let mut check = |commit: &crate::transactions::data::CommitResult| {
        let maintained = runtime
            .indexes
            .published_generation_for_commit(
                index_id,
                Some(&commit.commit.branch_id),
                commit.commit.commit_id,
                commit.commit.version_id,
            )
            .unwrap();
        // Cold reconstruction belongs to an isolated world: it must never
        // replace the maintained predecessor used by the next incremental step.
        let oracle = runtime.fork().unwrap();
        let cold = oracle.index_authority().build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: commit.commit.commit_id,
                branch_id: commit.commit.branch_id.clone(),
                index_ids: vec![index_id],
            },
        );
        assert!(cold.failed_indexes.is_empty());
        court
            .check_against_cold::<DerivedIndexMaintenanceDenialKind>(
                Arc::new(vec![maintained.entries.clone()]),
                Arc::new(vec![cold.generations[0].entries.clone()]),
                |left, right| left == right,
            )
            .unwrap();
    };
    check(&first);
    let mut previous = first;
    for ordinal in 0..16 {
        let candidate = candidate_for_update(
            execution,
            &runtime,
            entity,
            &format!("seeded-{}", (ordinal * 7) % 11),
        );
        let next = publish_native(execution, &runtime, candidate);
        check(&next);
        release_test_commit_snapshot(&runtime, &previous);
        previous = next;
    }
    release_test_commit_snapshot(&runtime, &previous);
}

pub(super) fn publish_native(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    candidate: crate::mvcc::PreparedRelationalCommitCandidate,
) -> crate::transactions::data::CommitResult {
    let work = candidate.prepared_index_maintenance_work().unwrap();
    assert_eq!(work.cold_record_slots, 0);
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("native candidate publishes");
    };
    runtime.settle_performed_publication(performed).unwrap()
}

#[test]
fn candidate_index_work_exhaustion_leaves_the_candidate_unpublished_and_retryable() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    assert!(runtime
        .index_authority()
        .build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: first.commit.commit_id,
                branch_id: first.commit.branch_id.clone(),
                index_ids: vec![index_id],
            }
        )
        .failed_indexes
        .is_empty());
    let transaction = transaction_for_update(&runtime, entity, "after");
    let (denied, _) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(Some(0)), |_| {
            runtime.prepare_branch_transaction(execution, transaction)
        })
        .unwrap();
    assert!(matches!(
        denied.unwrap_err(),
        crate::transactions::data::TransactionCommitError::Execution { .. }
    ));
    assert_eq!(runtime.history().immutable_commit_count(), 1);
    let candidate = candidate_for_update(execution, &runtime, entity, "after");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    assert!(candidate.prepared_index_maintenance_work().is_some());
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("retried candidate must publish");
    };
    let second = runtime.settle_performed_publication(performed).unwrap();
    assert!(runtime
        .indexes
        .published_generation_for_commit(
            index_id,
            Some(&BranchId("main".into())),
            commit_id,
            second.commit.version_id,
        )
        .is_some());
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn discarded_prepared_candidate_does_not_expose_index_generation() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    assert!(runtime
        .index_authority()
        .build_for_commit(
            execution,
            DerivedIndexBuildRequest {
                source_commit_id: first.commit.commit_id,
                branch_id: first.commit.branch_id.clone(),
                index_ids: vec![index_id],
            }
        )
        .failed_indexes
        .is_empty());
    let candidate = candidate_for_update(execution, &runtime, entity, "abandoned");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    runtime.discard_prepared_candidate(candidate).unwrap();
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .is_empty());
    release_test_commit_snapshot(&runtime, &first);
}

#[test]
fn candidate_patch_refresh_uses_prior_generation_without_cold_scan() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    let initial = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index_id],
        },
    );
    assert!(initial.failed_indexes.is_empty());
    let candidate = candidate_for_update(execution, &runtime, entity, "after");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    let work = candidate.prepared_index_maintenance_work().unwrap();
    assert_eq!(work.cold_record_slots, 0);
    assert_eq!(work.patch_records, 1);
    assert_eq!(work.entry_edits, 2);
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) = runtime
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("candidate must publish");
    };
    let second = runtime.settle_performed_publication(performed).unwrap();
    let generation = runtime
        .indexes
        .published_generation_for_commit(
            index_id,
            Some(&BranchId("main".into())),
            commit_id,
            second.commit.version_id,
        )
        .unwrap();
    assert_ne!(
        generation.generation_id,
        initial.generations[0].generation_id
    );
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

// Explicit fixture installation recovery, before an ordinary commit can maintain
// newly registered indexes on a populated head. Never called as a read fallback.
pub(super) fn reconstruct_installed_at_head(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
) {
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let ids = runtime
        .index_access()
        .definitions_snapshot()
        .iter()
        .map(|d| d.index_id)
        .collect();
    let result = runtime.index_authority().build_for_basis(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: basis.observation().commit_id().unwrap(),
            branch_id: BranchId("main".into()),
            index_ids: ids,
        },
        &basis,
    );
    assert!(
        result.failed_indexes.is_empty(),
        "explicit fixture recovery: {result:?}"
    );
}
