use super::*;
use crate::facade::indexes::{
    DerivedIndexEntries, DerivedIndexMaintenanceBudget, DerivedIndexMaintenanceDenialKind,
};
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
        .index_id
}

fn candidate_for_update(
    runtime: &RelationalRuntime,
    entity_id: crate::facade::identity::EntityId,
    name: &str,
) -> crate::mvcc::PreparedRelationalCommitCandidate {
    let mut transaction = test_owner_begin_transaction_for_main(runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("candidate-index-update").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id,
                    fields: name_field_patch(name),
                }),
            )),
        )
        .unwrap();
    runtime.prepare_branch_transaction(transaction).unwrap()
}

fn budget(cold_slots: usize) -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: 100_000,
        maximum_cold_record_slots: cold_slots,
        maximum_derived_rows: 100_000,
    }
}

#[test]
fn expired_candidate_denies_index_preflight_with_lifetime_not_missing_generation() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(declared_aspect_schema_registry(
            CascadeDeletePolicy::CascadeDeleteRelations,
        ))
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 8,
            max_transaction_overlay_bytes: 1_048_576,
            max_transaction_footprint_loci: 1_024,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 0,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    let index_id = index(&runtime);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("expired-index-candidate"))
        .unwrap();
    let mut candidate = runtime.prepare_branch_transaction(transaction).unwrap();

    let denied = runtime
        .index_authority()
        .prepare_for_candidate(&mut candidate, &[index_id], None, budget(100))
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::CandidateLifetimeExpired {
            maximum_lifetime_millis: 0,
        }
    );
    assert_eq!(runtime.index_access().generations_snapshot().len(), 0);
    assert_eq!(runtime.history().immutable_commit_count(), 0);
}

#[test]
fn candidate_indexes_deny_before_effect_then_publish_exact_cold_generation_on_settlement() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    let mut candidate = candidate_for_update(&runtime, entity, "after");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    let denied = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(0),
        )
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    );
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .is_empty());

    let work = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(100),
        )
        .unwrap();
    assert_eq!(work.generation_publications_reserved, 1);
    assert!(work.cold_record_slots > 0);
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .is_empty());
    let duplicate = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(100),
        )
        .unwrap_err();
    assert_eq!(
        duplicate.kind,
        DerivedIndexMaintenanceDenialKind::CandidateIndexesAlreadyPrepared
    );

    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
    else {
        panic!("candidate must publish");
    };
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .is_empty());
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
    let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
        panic!("entity field index");
    };
    let after_key =
        AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value("after"));
    let before_key =
        AuthoritativeFieldComparisonKey::from_aspect_value(&string_aspect_value("before"));
    assert_eq!(entries.get(&after_key).unwrap().get(0), Some(&entity));
    assert!(entries.get(&before_key).is_none());
    release_test_commit_snapshot(&runtime, &first);
    release_test_commit_snapshot(&runtime, &second);
}

#[test]
fn candidate_index_work_exhaustion_leaves_the_candidate_unpublished_and_retryable() {
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    let mut candidate = candidate_for_update(&runtime, entity, "after");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    let denied = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            DerivedIndexMaintenanceBudget {
                maximum_work_units: 0,
                maximum_cold_record_slots: 0,
                maximum_derived_rows: 0,
            },
        )
        .unwrap_err();
    assert_eq!(
        denied.kind,
        DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
    );
    assert!(runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .is_empty());
    runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(100),
        )
        .unwrap();
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
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
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    let mut candidate = candidate_for_update(&runtime, entity, "abandoned");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(100),
        )
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
    let runtime = runtime_with_index_field_aspects();
    let first = create_entity_outcome(&runtime, "before");
    let entity = changed_entities(&first)[0];
    let index_id = index(&runtime);
    let initial = runtime
        .index_authority()
        .build_for_commit(DerivedIndexBuildRequest {
            source_commit_id: first.commit.commit_id,
            branch_id: BranchId("main".into()),
            index_ids: vec![index_id],
        });
    assert!(initial.failed_indexes.is_empty());
    let mut candidate = candidate_for_update(&runtime, entity, "after");
    let commit_id = candidate
        .index_preparation_roots()
        .unwrap()
        .1
        .commit_id()
        .unwrap();
    let work = runtime
        .index_authority()
        .prepare_for_candidate(
            &mut candidate,
            &[index_id],
            Some(&first.snapshot),
            budget(0),
        )
        .unwrap();
    assert_eq!(work.cold_record_slots, 0);
    assert_eq!(work.patch_records, 1);
    assert_eq!(work.entry_edits, 2);
    let crate::mvcc::RelationalPublicationOutcome::Performed(performed) =
        runtime.publication_port().compare_and_publish(candidate)
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
