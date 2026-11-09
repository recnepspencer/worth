//! Branch-local endpoint retirement meaning across actual checkpoint recovery.
use super::branch_history_publications::*;
use super::branch_history_recovery::{checkpoint, current_head, recover};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    history::BranchId,
    identity::{KindId, RelationId},
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::*,
    transactions::*,
};
use worth_testkit_relational::{
    aspect_storage::publications::{changed_entities, release_test_commit_snapshot},
    durable_medium::TestDurableMedium,
};
fn relation_digest(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    branch: &BranchId,
    relation: RelationId,
    filter: Option<&worth_relational::facade::runtime::ProjectionAspectFilter>,
) -> worth_relational::facade::history::AspectHistoryDigest {
    runtime
        .history()
        .relation_aspect_history_with_trace(request, branch, relation, filter)
        .unwrap()
        .aspect_history_digest()
}
#[test]
fn durability_contract_recovery_preserves_branch_local_endpoint_deletion_retirement_histories() {
    let host_request = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
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
                relation_integrity: worth_relational::facade::schema::RelationIntegrityDeclarations::new(
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    vec![worth_relational::facade::schema::EndpointDeletionIntegrityDeclaration {
                        contract_id: "require_retirement".into(),
                        mode: worth_relational::facade::schema::EndpointDeletionIntegrityMode::RequireRelationRetirement,
                    }],
                ),
            })
        })
        .unwrap();
    let medium = TestDurableMedium::new();
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry.clone())
        .cascade_delete_policy(CascadeDeletePolicy::RetainDanglingForAudit)
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration");
    let created_source = create_with_fields(
        execution,
        &runtime,
        "source",
        &BranchId("main".into()),
        AspectFieldPatch::default(),
    );
    let source = changed_entities(&created_source)[0];
    release_test_commit_snapshot(&runtime, &created_source);
    let created_target = create_with_fields(
        execution,
        &runtime,
        "target",
        &BranchId("main".into()),
        AspectFieldPatch::default(),
    );
    let target = changed_entities(&created_target)[0];
    release_test_commit_snapshot(&runtime, &created_target);
    let relation_outcome = relation_with_fields(
        execution,
        &runtime,
        source,
        target,
        "retained",
        AspectFieldPatch::default(),
    );
    let relation = changed_relations(&relation_outcome)[0];
    fork(execution, &runtime, "feature");
    let _main_delete = delete_entity(execution, &runtime, source);
    let _feature_update = transaction(&runtime, &BranchId("feature".into()))
        .commit(execution, &runtime)
        .unwrap();

    let expected_main_digest = relation_digest(
        execution,
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let expected_feature_digest = relation_digest(
        execution,
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
            current_head(&runtime, &BranchId("main".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            worth_relational::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    let expected_feature_inspection = runtime
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("feature".to_string()),
            current_head(&runtime, &BranchId("feature".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            worth_relational::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    checkpoint(execution, &runtime, &medium);
    let receiver = RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .cascade_delete_policy(CascadeDeletePolicy::RetainDanglingForAudit)
        .build_durable(medium.attachment())
        .unwrap();
    let (outcome, recovered) = recover(execution, &medium, receiver);

    let recovered_main_digest = relation_digest(
        execution,
        &recovered,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let recovered_feature_digest = relation_digest(
        execution,
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
            current_head(&recovered, &BranchId("main".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            worth_relational::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");
    let recovered_feature_inspection = recovered
        .inspect_what_happened()
        .inspect_historical_record(
            execution,
            &BranchId("feature".to_string()),
            current_head(&recovered, &BranchId("feature".to_string()))
                .unwrap()
                .version_id,
            RecordRef::Relation(relation),
            worth_relational::facade::inspection::HistoricalInspectionMode::RetainedOnly,
        )
        .expect("read request remains live");

    assert_eq!(
        outcome.latest_commit,
        runtime.history().historical_latest_commit()
    );
    assert_eq!(expected_main_digest, recovered_main_digest);
    assert_eq!(expected_feature_digest, recovered_feature_digest);
    assert_eq!(expected_main_inspection, recovered_main_inspection);
    assert_eq!(expected_feature_inspection, recovered_feature_inspection);
}
