use crate::facade::mvcc::{
    PreparedRelationalChangeSummaryBudget as Budget,
    PreparedRelationalChangeSummaryDenial as Denial, PreparedRelationalRelationEndpoints,
};
use crate::facade::publication::RecordStructuralChange;
use crate::facade::transactions::{
    DeleteRelationIntent, EntityReference, MutationIntent, RecordRef, RelationMutationIntent,
    UpdateEntityFieldsIntent, UpdateRelationEndpointsIntent, WorkerIntentBatch,
};
use crate::tests::support::*;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};

const GENEROUS: Budget = Budget {
    max_records: 16,
    max_aspect_scopes: 64,
};

#[test]
fn prepared_summary_reads_exact_whole_scalar_scope_without_publishing() {
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&runtime, "summary-before");
    let before = test_owner_main_basis(&runtime).unwrap().observation();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("summary-update").push(MutationIntent::Entity(
                crate::facade::transactions::EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: name_field_patch("summary-after"),
                    },
                ),
            )),
        )
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let summary = runtime
        .preparation_port()
        .summarize_prepared_candidate(&candidate, GENEROUS)
        .unwrap();

    assert_eq!(
        summary.runtime_instance_id,
        before.descriptor().runtime_instance_id()
    );
    assert_eq!(summary.branch_id, *before.descriptor().branch_id());
    assert_eq!(summary.before_version, before.version_id());
    assert_eq!(summary.before_commit_id, before.commit_id());
    assert!(summary.after_version > summary.before_version);
    let record = summary
        .records
        .iter()
        .find(|record| record.target == RecordRef::Entity(entity))
        .unwrap();
    assert_eq!(record.structural_change, RecordStructuralChange::Updated);
    assert!(record
        .aspect_scopes
        .iter()
        .any(|scope| { scope.aspect_key.as_str() == "name" && scope.field_path.is_none() }));
    assert_eq!(record.before_endpoints, None);
    assert_eq!(record.after_endpoints, None);
    assert_eq!(
        test_owner_main_basis(&runtime)
            .unwrap()
            .observation()
            .version_id(),
        before.version_id()
    );
    runtime.discard_prepared_candidate(candidate).unwrap();
}

#[test]
fn prepared_summary_preserves_exact_struct_field_scope() {
    let runtime = AspectSchemaFixture {
        entity_aspects: vec![entity_summary_struct_aspect(
            aspect_key("summary"),
            field_key("summary"),
        )],
        ..AspectSchemaFixture::default()
    }
    .build_runtime();
    let entity = super::struct_field_patch_authority::create_entity_with_summary_fields(
        &runtime,
        "summary-struct",
        "before",
        "open",
        false,
        false,
    );
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("summary-struct-field").push(MutationIntent::Entity(
                crate::facade::transactions::EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: crate::facade::transactions::AspectFieldPatch::from_locator(
                            crate::facade::transactions::planned_single_field_locator(
                                AspectKey::new("summary").unwrap(),
                                FieldKey::new("title").unwrap(),
                            ),
                            AspectValue::String("after".into()),
                        ),
                    },
                ),
            )),
        )
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let summary = runtime
        .preparation_port()
        .summarize_prepared_candidate(&candidate, GENEROUS)
        .unwrap();
    let record = summary
        .records
        .iter()
        .find(|record| record.target == RecordRef::Entity(entity))
        .unwrap();
    assert_eq!(record.aspect_scopes.len(), 1);
    assert_eq!(record.aspect_scopes[0].aspect_key.as_str(), "summary");
    assert_eq!(
        record.aspect_scopes[0]
            .field_path
            .as_ref()
            .unwrap()
            .fields()[0]
            .as_str(),
        "title"
    );
    runtime.discard_prepared_candidate(candidate).unwrap();
}

#[test]
fn prepared_summary_uses_before_and_after_relation_roots_for_rewire() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "summary-source");
    let old_target = create_entity(&runtime, "summary-old-target");
    let new_target = create_entity(&runtime, "summary-new-target");
    let relation = create_relation(&runtime, source, old_target, "summary-edge");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("summary-rewire").push(MutationIntent::Relation(
                RelationMutationIntent::UpdateEndpoints(UpdateRelationEndpointsIntent {
                    relation_id: relation,
                    kind_id: KindId(2),
                    source: EntityReference::Existing(source),
                    target: EntityReference::Existing(new_target),
                }),
            )),
        )
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let summary = runtime
        .preparation_port()
        .summarize_prepared_candidate(&candidate, GENEROUS)
        .unwrap();
    let record = summary
        .records
        .iter()
        .find(|record| record.target == RecordRef::Relation(relation))
        .unwrap();
    assert_eq!(
        record.before_endpoints,
        Some(PreparedRelationalRelationEndpoints {
            kind_id: KindId(2),
            source,
            target: old_target,
        })
    );
    assert_eq!(
        record.after_endpoints,
        Some(PreparedRelationalRelationEndpoints {
            kind_id: KindId(2),
            source,
            target: new_target,
        })
    );
    runtime.discard_prepared_candidate(candidate).unwrap();
}

#[test]
fn prepared_relation_delete_has_only_before_live_endpoints() {
    let runtime = runtime_with_test_schema();
    let source = create_entity(&runtime, "summary-delete-source");
    let target = create_entity(&runtime, "summary-delete-target");
    let relation = create_relation(&runtime, source, target, "summary-delete-edge");
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("summary-delete").push(MutationIntent::Relation(
                RelationMutationIntent::Delete(DeleteRelationIntent {
                    relation_id: relation,
                }),
            )),
        )
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    let summary = runtime
        .preparation_port()
        .summarize_prepared_candidate(&candidate, GENEROUS)
        .unwrap();
    let record = summary
        .records
        .iter()
        .find(|record| record.target == RecordRef::Relation(relation))
        .unwrap();
    assert_eq!(record.structural_change, RecordStructuralChange::Deleted);
    assert_eq!(
        record.before_endpoints,
        Some(PreparedRelationalRelationEndpoints {
            kind_id: KindId(2),
            source,
            target,
        })
    );
    assert_eq!(record.after_endpoints, None);
    runtime.discard_prepared_candidate(candidate).unwrap();
}

#[test]
fn summary_budget_and_foreign_owner_deny_without_consuming_candidate() {
    let runtime = runtime_with_test_schema();
    let foreign = runtime_with_test_schema();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("summary-denial"))
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    assert_eq!(
        foreign
            .preparation_port()
            .summarize_prepared_candidate(&candidate, GENEROUS),
        Err(Denial::ForeignCandidate)
    );
    assert_eq!(
        runtime.preparation_port().summarize_prepared_candidate(
            &candidate,
            Budget {
                max_records: 0,
                max_aspect_scopes: 0,
            },
        ),
        Err(Denial::RecordBudgetExceeded)
    );
    assert_eq!(
        runtime.preparation_port().summarize_prepared_candidate(
            &candidate,
            Budget {
                max_records: 16,
                max_aspect_scopes: 0,
            },
        ),
        Err(Denial::AspectScopeBudgetExceeded)
    );
    assert!(!runtime
        .preparation_port()
        .summarize_prepared_candidate(&candidate, GENEROUS)
        .unwrap()
        .records
        .is_empty());
    runtime.discard_prepared_candidate(candidate).unwrap();
}

#[test]
fn expired_candidate_summary_is_typed_and_cannot_retain_stale_scope() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(crate::facade::config::PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 1,
            max_active_snapshot_handles: 8,
            max_transaction_overlay_bytes: 1_048_576,
            max_transaction_footprint_loci: 1_024,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 1,
            candidate_max_lifetime_millis: 0,
            max_prepared_root_bytes: 268_435_456,
        })
        .build();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(batch_create("expired-summary"))
        .unwrap();
    let candidate = runtime.prepare_branch_transaction(transaction).unwrap();
    assert_eq!(
        runtime
            .preparation_port()
            .summarize_prepared_candidate(&candidate, GENEROUS),
        Err(Denial::CandidateLifetimeExpired)
    );
    runtime.discard_prepared_candidate(candidate).unwrap();
}
