use super::*;
use crate::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
    DerivedIndexEntries,
};

#[test]
fn entity_field_certification_rejects_a_missing_candidate() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "parity-candidate");
    let entity_id = changed_entities(&created)[0];
    let field_locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(75),
        name: "entity.name.parity-red-control".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field_locator.clone(),
        },
        branch_scoped: false,
    });
    build_entity_field_generation(execution, &runtime, index.index_id);
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::EntityField(entries) = &mut generation.entries else {
                panic!("entity-field generation expected");
            };
            entries.clear();
        });
    let snapshot = runtime.visibility_authority().snapshot();
    let request = || {
        entity_field_request(
            snapshot.clone(),
            index.index_id,
            field_locator.clone(),
            string_aspect_value("parity-candidate"),
        )
    };

    let production = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    assert!(production.candidate_entity_ids().is_empty());
    assert_ne!(production.candidate_entity_ids(), &[entity_id]);
    let denial = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        BoundedEntityFieldLookupDenialKind::StorageParityMismatch
    );
}

#[test]
fn bounded_lookup_caps_ordinary_work_and_certifies_storage_parity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let entities = (0..32)
        .map(|index| {
            let outcome = create_entity_outcome(&runtime, &format!("candidate-{index}"));
            changed_entities(&outcome)[0]
        })
        .collect::<Vec<_>>();
    for entity in &entities {
        update_entity(&runtime, *entity, "shared");
    }
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(71),
        name: "entity.name.bounded".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
    let snapshot = runtime.visibility_authority().snapshot();
    let request = || {
        BoundedEntityFieldLookupRequest::new(
            snapshot.clone(),
            index.index_id,
            KindId(1),
            aspect_field_locator(aspect_key("name"), field_key("name")),
            string_aspect_value("shared"),
            2,
        )
        .unwrap()
    };

    let production = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Production,
            execution,
        )
        .unwrap();
    let certified = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            request(),
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();

    assert_eq!(production.candidate_entity_ids().len(), 2);
    assert_eq!(production.examined_entry_count(), 2);
    assert!(production.overflowed());
    assert_eq!(
        production.candidate_entity_ids(),
        certified.candidate_entity_ids()
    );
    assert_eq!(production.overflowed(), certified.overflowed());
}

#[test]
fn corrupt_bucket_rejects_a_large_unrelated_field_before_encoding_it() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let requested = create_entity_outcome(&runtime, "small");
    let unrelated = create_entity_outcome(&runtime, "large");
    let unrelated_id = changed_entities(&unrelated)[0];
    update_entity(&runtime, unrelated_id, &"x".repeat(32 * 1024));
    let locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(76),
        name: "entity.name.corrupt-large-value".to_owned(),
        kind: DerivedIndexKind::EntityField {
            field_locator: locator.clone(),
        },
        branch_scoped: false,
    });
    build_entity_field_generation(execution, &runtime, index.index_id);
    let requested_key = crate::facade::storage::AuthoritativeFieldComparisonKey::from_aspect_value(
        &string_aspect_value("small"),
    );
    runtime
        .indexes
        .corrupt_latest_generation(index.index_id, |generation| {
            let DerivedIndexEntries::EntityField(entries) = &mut generation.entries else {
                panic!("entity-field generation expected");
            };
            let rows = entries.get_mut(&requested_key).expect("small entry exists");
            rows.remove(0);
            rows.insert(0, unrelated_id);
        });
    let request = BoundedEntityFieldLookupRequest::new(
        runtime.visibility_authority().snapshot(),
        index.index_id,
        KindId(1),
        locator,
        string_aspect_value("small"),
        2,
    )
    .unwrap();
    assert_ne!(changed_entities(&requested)[0], unrelated_id);
    let denial = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production, execution)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
    );
    assert_eq!(denial.examined_entry_count(), 1);
}

#[test]
fn bounded_lookup_rejects_empty_budgets_and_nonexact_generations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "alpha");
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(72),
        name: "entity.name.exact-generation".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: aspect_field_locator(aspect_key("name"), field_key("name")),
        },
        branch_scoped: false,
    });
    runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: created.commit.commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    let empty = BoundedEntityFieldLookupRequest::new(
        created.snapshot.clone(),
        index.index_id,
        KindId(1),
        aspect_field_locator(aspect_key("name"), field_key("name")),
        string_aspect_value("alpha"),
        0,
    )
    .unwrap_err();
    assert_eq!(
        empty.kind(),
        BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
    );

    create_entity_outcome(&runtime, "later");
    let later_snapshot = runtime.visibility_authority().snapshot();
    let request = BoundedEntityFieldLookupRequest::new(
        later_snapshot,
        index.index_id,
        KindId(1),
        aspect_field_locator(aspect_key("name"), field_key("name")),
        string_aspect_value("alpha"),
        2,
    )
    .unwrap();
    let denial = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(request, BoundedIndexParityMode::Production, execution)
        .unwrap_err();
    assert_eq!(
        denial.kind(),
        BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable
    );
}

#[test]
fn bounded_lookup_rebuilds_exact_historical_generation_after_truth_advances() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_index_field_aspects();
    let created = create_entity_outcome(&runtime, "historical");
    let entity_id = changed_entities(&created)[0];
    let historical_snapshot = created.snapshot.clone();
    let historical_commit_id = created.commit.commit_id;
    let historical_version_id = created.version_id;
    let historical_value = string_aspect_value("historical");
    let field_locator = aspect_field_locator(aspect_key("name"), field_key("name"));
    let index = runtime.index_authority().register(DerivedIndexDefinition {
        index_id: DerivedIndexId(73),
        name: "entity.name.historical-rebuild".to_string(),
        kind: DerivedIndexKind::EntityField {
            field_locator: field_locator.clone(),
        },
        branch_scoped: false,
    });

    update_entity(&runtime, entity_id, "current");
    assert_ne!(historical_version_id, runtime.current_version_id());
    let historical_read = runtime
        .read_truth()
        .read_version(execution, historical_version_id);
    let historical_record = historical_read.get_entity(entity_id).unwrap();
    assert_eq!(
        crate::visibility::materialization::read_records::entity_query_locus_comparison_key(
            historical_record,
            &field_locator,
        ),
        Some(
            crate::storage::data::AuthoritativeFieldComparisonKey::from_aspect_value(
                &historical_value,
            )
        )
    );
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id: historical_commit_id,
            branch_id: BranchId("main".to_string()),
            index_ids: vec![index.index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());

    let request = BoundedEntityFieldLookupRequest::new(
        historical_snapshot,
        index.index_id,
        KindId(1),
        field_locator,
        historical_value,
        2,
    )
    .unwrap();
    let result = runtime
        .index_access()
        .execute_bounded_entity_field_lookup(
            request,
            BoundedIndexParityMode::Certification,
            execution,
        )
        .unwrap();

    assert_eq!(result.candidate_entity_ids(), &[entity_id]);
    assert!(!result.overflowed());
}

fn build_entity_field_generation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    index_id: DerivedIndexId,
) {
    let source_commit_id = runtime.history().latest_commit().unwrap().commit_id;
    let build = runtime.index_authority().build_for_commit(
        execution,
        DerivedIndexBuildRequest {
            source_commit_id,
            branch_id: BranchId("main".to_owned()),
            index_ids: vec![index_id],
        },
    );
    assert!(build.failed_indexes.is_empty());
}

fn entity_field_request(
    snapshot: crate::facade::snapshots::SnapshotHandle,
    index_id: DerivedIndexId,
    field_locator: worth_foundational::facade::AspectFieldLocator,
    value: worth_foundational::facade::AspectValue,
) -> BoundedEntityFieldLookupRequest {
    BoundedEntityFieldLookupRequest::new(snapshot, index_id, KindId(1), field_locator, value, 2)
        .unwrap()
}

#[path = "bounded_entity_field_lookup/diverged_heads.rs"]
mod diverged_heads;
