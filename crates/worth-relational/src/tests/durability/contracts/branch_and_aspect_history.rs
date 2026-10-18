use super::*;
use crate::durability::data::RecoveryVerificationMode as RecoveryMode;

#[test]
fn durability_contract_recovery_rebuilds_branch_heads_and_latest_commit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = persisted_runtime_with_test_schema();
    let main = create_entity_outcome(execution, &runtime, "main-a");
    create_branch_from_main(&runtime, "feature");
    let feature = create_entity_outcome_on_branch(
        execution,
        &runtime,
        "feature-a",
        BranchId("feature".to_string()),
    );
    let (outcome, recovered) =
        checkpoint_and_recover_with(execution, &runtime, persisted_runtime_with_test_schema);

    assert_eq!(outcome.recovered_commits, 2);
    assert_eq!(outcome.latest_commit, Some(feature.commit.clone()));
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("feature".to_string())),
        Some(feature.commit.clone())
    );
    assert_eq!(
        recovered
            .history()
            .branch_head(&BranchId("main".to_string())),
        Some(main.commit.clone())
    );
}

#[test]
fn durability_contract_recovery_preserves_aspect_bearing_patch_truth_and_history() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
    let created = create_entity_outcome(execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let updated = update_entity(execution, &runtime, entity, "after");
    let expected_history =
        runtime
            .history()
            .entity_aspect_history(&BranchId("main".to_string()), entity, None);
    let expected_digest = runtime
        .history()
        .entity_aspect_history_with_trace(&BranchId("main".to_string()), entity, None)
        .aspect_history_digest();
    let expected_envelope = runtime
        .replay()
        .canonical_commit_envelope(updated.commit.commit_id)
        .unwrap();
    let (outcome, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations)
    });

    let recovered_history =
        recovered
            .history()
            .entity_aspect_history(&BranchId("main".to_string()), entity, None);
    let recovered_digest = recovered
        .history()
        .entity_aspect_history_with_trace(&BranchId("main".to_string()), entity, None)
        .aspect_history_digest();
    let recovered_replay = recovered.replay();
    let recovered_envelope = recovered_replay
        .canonical_commit_envelope(updated.commit.commit_id)
        .unwrap();

    assert_eq!(outcome.latest_commit, Some(updated.commit.clone()));
    assert_eq!(expected_history, recovered_history);
    assert_eq!(expected_digest, recovered_digest);
    assert_eq!(
        expected_envelope.patch.authoritative_record_patches,
        recovered_envelope.patch.authoritative_record_patches
    );
    assert_eq!(
        recovered_envelope.patch.authoritative_record_patches[0].authoritative_changed_aspects(),
        ordered_aspect_keys([aspect_key("name")])
    );
    assert!(!recovered_envelope.patch.authoritative_record_patches[0].contains_opaque_aspect);
}

#[test]
fn durability_contract_recovery_preserves_relation_aspect_history_for_retained_audit_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime =
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit);
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome = create_relation_outcome(execution, &runtime, source, target, "r-audit");
    let relation = changed_relations(&relation_outcome)[0];
    let deleted = delete_entity(execution, &runtime, source);
    let expected_history =
        runtime
            .history()
            .relation_aspect_history(&BranchId("main".to_string()), relation, None);
    let expected_digest = runtime
        .history()
        .relation_aspect_history_with_trace(&BranchId("main".to_string()), relation, None)
        .aspect_history_digest();
    let (outcome, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        persisted_runtime_with_declared_aspect_schema(CascadeDeletePolicy::RetainDanglingForAudit)
    });

    let recovered_history =
        recovered
            .history()
            .relation_aspect_history(&BranchId("main".to_string()), relation, None);
    let recovered_digest = recovered
        .history()
        .relation_aspect_history_with_trace(&BranchId("main".to_string()), relation, None)
        .aspect_history_digest();

    assert_eq!(outcome.latest_commit, Some(deleted.commit.clone()));
    assert_eq!(expected_history, recovered_history);
    assert_eq!(expected_digest, recovered_digest);
    assert_eq!(recovered_history.len(), 2);
    assert_direct_history_origin_invariants(&recovered_history, RecordRef::Relation(relation));
    assert_eq!(
        recovered_history[0].origin.changed_aspects,
        ordered_aspect_keys([
            aspect_key("label"),
            aspect_key("lifecycle"),
            aspect_key("source"),
            aspect_key("target"),
        ])
    );
    assert_eq!(
        recovered_history[1].origin.changed_aspects,
        ordered_aspect_keys([aspect_key("lifecycle")])
    );
}

#[test]
fn durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec![crate::schema::data::EndpointDeletionIntegrityDeclaration {
                        contract_id: "require_retirement".into(),
                        mode: crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
                    }],
                ),
            })
        })
        .unwrap();
    let store_layout = DurableStoreLayout {
        root_path: unique_test_store_path("worth-relational-endpoint-retirement-recovery"),
        segment_commit_capacity: 2,
    };
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout.clone())
        .cascade_delete_policy(CascadeDeletePolicy::RetainDanglingForAudit)
        .build();
    let source = create_entity(execution, &runtime, "source");
    let target = create_entity(execution, &runtime, "target");
    let relation_outcome = create_relation_outcome(execution, &runtime, source, target, "retained");
    let relation = changed_relations(&relation_outcome)[0];
    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();
    let _main_delete = delete_entity(execution, &runtime, source);
    let _feature_update = update_entity_on_branch(
        execution,
        &runtime,
        target,
        "feature-target",
        BranchId("feature".to_string()),
    );

    let expected_main_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let expected_feature_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("feature".to_string()),
        relation,
        None,
    );
    let expected_main_inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("main".to_string()),
            runtime
                .history()
                .branch_head(&BranchId("main".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    let expected_feature_inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("feature".to_string()),
            runtime
                .history()
                .branch_head(&BranchId("feature".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    runtime
        .durability_authority()
        .checkpoint(execution)
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan(execution, RecoveryMode::NormalRecoveryVerification)
        .unwrap();
    let mut recovered = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .durability_mode(DurabilityMode::PersistedSegmentedLocalFs)
        .durable_store_layout(store_layout)
        .cascade_delete_policy(CascadeDeletePolicy::RetainDanglingForAudit)
        .build();
    let outcome = recovered
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();

    let recovered_main_digest = relation_aspect_history_digest_on_branch(
        &recovered,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let recovered_feature_digest = relation_aspect_history_digest_on_branch(
        &recovered,
        &BranchId("feature".to_string()),
        relation,
        None,
    );
    let recovered_main_inspection = recovered
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("main".to_string()),
            recovered
                .history()
                .branch_head(&BranchId("main".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    let recovered_feature_inspection = recovered
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("feature".to_string()),
            recovered
                .history()
                .branch_head(&BranchId("feature".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    assert_eq!(outcome.latest_commit, runtime.history().latest_commit());
    assert_eq!(expected_main_digest, recovered_main_digest);
    assert_eq!(expected_feature_digest, recovered_feature_digest);
    assert_eq!(expected_main_inspection, recovered_main_inspection);
    assert_eq!(expected_feature_inspection, recovered_feature_inspection);
}
