use crate::facade::storage::RecordLifecycleState;
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn relation_integrity_commit_boundary_rejects_endpoint_delete_with_live_relations() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(
                error.code(),
                DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
                "unexpected relation-integrity denial: {error:?}"
            );
            match error.class {
                crate::transactions::data::ConflictClass::InvariantViolation {
                    fields:
                        crate::validation::data::InvariantViolationFields::RelationEndpointDeletionIntegrity {
                            contract_id,
                            relation_kind_id,
                            entity_id,
                            mode,
                            ..
                        },
                    ..
                } => {
                    assert_eq!(contract_id.as_str(), "endpoint_delete");
                    assert_eq!(relation_kind_id, KindId(2));
                    assert_eq!(entity_id, source);
                    assert_eq!(
                        mode,
                        crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations
                    );
                }
                other => {
                    panic!("expected typed endpoint deletion invariant conflict, got {other:?}")
                }
            }
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_rejects_replace_when_retained_relation_keeps_live_endpoint_dependency(
) {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("replace-source").push(MutationIntent::Entity(
            EntityMutationIntent::Replace(ReplaceEntityIntent {
                entity_id: source,
                replacement: crate::transactions::data::EntitySpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(1),
                    client_key: crate::symbols::data::ClientKey::raw("source-replacement"),
                    fields: crate::transactions::data::AspectFieldPatch::default(),
                },
            }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(
                error.code(),
                DiagnosticCode::RelationEndpointDeletionIntegrityViolation,
                "unexpected replace denial: {error:?}"
            );
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_requires_relation_deletion_in_same_commit_under_retain_policy(
) {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(
                error.code(),
                DiagnosticCode::RelationEndpointDeletionIntegrityViolation
            );
            assert!(error
                .detail()
                .contains("requires deleting dependent relations in the same commit"));
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_allows_relation_deletion_in_same_commit_under_cascade_policy()
{
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let (source, _target, relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let deleted = delete_entity(&runtime, source);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted.snapshot)
        .unwrap();

    assert!(read.get_relation(relation).is_none());
}

#[test]
fn relation_integrity_commit_boundary_allows_relation_retirement_when_policy_retains_for_audit() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let deleted = delete_entity(&runtime, source);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted.snapshot)
        .unwrap();
    let relation = read.get_relation(relation).unwrap();

    assert_eq!(
        relation.lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}

#[test]
fn historical_relation_lifecycle_does_not_reveal_future_audit_retention_after_recovery() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, relation) =
        create_endpoint_deletion_relation_fixture(&runtime, "historical-lifecycle");
    let live_version = runtime.current_version_id();
    let retired = delete_entity(&runtime, source);
    create_entity(&runtime, "after-audit-retirement");
    let post_retirement_version = runtime.current_version_id();

    assert_relation_lifecycle_at_version(
        &runtime,
        relation,
        live_version,
        RecordLifecycleState::Live,
        None,
    );
    assert_relation_lifecycle_at_version(
        &runtime,
        relation,
        retired.version_id,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
    assert_relation_lifecycle_at_version(
        &runtime,
        relation,
        post_retirement_version,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );

    let (_, recovered) = checkpoint_and_recover_with(&runtime, || {
        endpoint_deletion_runtime(
            crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
            CascadeDeletePolicy::RetainDanglingForAudit,
        )
    });
    assert_relation_lifecycle_at_version(
        &recovered,
        relation,
        live_version,
        RecordLifecycleState::Live,
        None,
    );
    assert_relation_lifecycle_at_version(
        &recovered,
        relation,
        retired.version_id,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
    assert_relation_lifecycle_at_version(
        &recovered,
        relation,
        post_retirement_version,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
}

fn assert_relation_lifecycle_at_version(
    runtime: &crate::runtime::RelationalRuntime,
    relation: crate::identity::data::RelationId,
    version: crate::identity::data::VersionId,
    lifecycle: RecordLifecycleState,
    retired_at: Option<crate::identity::data::VersionId>,
) {
    let view = runtime.read_truth().read_version(version);
    let record = view
        .get_relation(relation)
        .expect("audit-retained relation remains materialized");
    assert_eq!(record.lifecycle, lifecycle);
    assert_eq!(record.retired_at_version, retired_at);
}

#[test]
fn relation_integrity_commit_boundary_rejects_relation_retirement_under_cascade_policy() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let (source, _target, _relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn
        .commit(&runtime, Allocation::SystemAllocation)
        .unwrap_err();
    match error {
        TransactionCommitError::Conflict { error, .. } => {
            assert_eq!(
                error.code(),
                DiagnosticCode::RelationEndpointDeletionIntegrityViolation
            );
            assert!(error
                .detail()
                .contains("requires audit-retained relation retirement"));
        }
        other => panic!("expected conflict, got {:?}", other),
    }
}

#[test]
fn relation_integrity_commit_boundary_allows_opposite_endpoint_delete_after_relation_retirement() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, target, relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    delete_entity(&runtime, source);
    let deleted_target = delete_entity(&runtime, target);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted_target.snapshot)
        .unwrap();
    let relation = read.get_relation(relation).unwrap();

    assert_eq!(
        relation.lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}

#[test]
fn relation_integrity_endpoint_deletion_history_stays_branch_local_under_divergence() {
    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, target, relation) = create_endpoint_deletion_relation_fixture(&runtime, "live");

    runtime
        .history_authority()
        .fork_branch_from(
            BranchId("feature".to_string()),
            &BranchId("main".to_string()),
        )
        .unwrap();

    let main_delete = delete_entity(&runtime, source);
    let _feature_update = update_entity_on_branch(
        &runtime,
        target,
        "feature-target",
        BranchId("feature".to_string()),
    );

    let main_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("main".to_string()),
        relation,
        None,
    );
    let feature_digest = relation_aspect_history_digest_on_branch(
        &runtime,
        &BranchId("feature".to_string()),
        relation,
        None,
    );
    let main_head_version = runtime
        .history()
        .branch_head(&BranchId("main".to_string()))
        .unwrap()
        .version_id;
    let feature_head_version = runtime
        .history()
        .branch_head(&BranchId("feature".to_string()))
        .unwrap()
        .version_id;
    let main_inspection = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("main".to_string()),
        main_head_version,
        RecordRef::Relation(relation),
        crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
    );
    let feature_inspection = runtime.inspect_what_happened().inspect_historical_record(
        &BranchId("feature".to_string()),
        feature_head_version,
        RecordRef::Relation(relation),
        crate::facade::inspection::HistoricalInspectionMode::RetainedOnly,
    );

    assert_eq!(main_delete.version_id, main_head_version);
    assert_eq!(main_digest.entry_count, 2);
    assert!(feature_digest.entry_count < main_digest.entry_count);
    assert_eq!(
        main_inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("main".to_string()))
    );
    assert_eq!(
        feature_inspection
            .aspect_history_observation
            .as_ref()
            .map(|observation| observation.query_result.trace.branch_id.clone()),
        Some(BranchId("feature".to_string()))
    );
    let main_read = runtime
        .read_truth()
        .read_snapshot(&main_delete.snapshot)
        .unwrap();
    assert_eq!(
        main_read.get_relation(relation).unwrap().lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}
