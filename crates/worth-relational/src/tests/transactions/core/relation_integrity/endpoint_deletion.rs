use crate::facade::storage::RecordLifecycleState;
use crate::tests::support::*;
use worth_execution::ExecutionAllocationPolicy as Allocation;

#[test]
fn relation_integrity_commit_boundary_rejects_endpoint_delete_with_live_relations() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn.commit(execution, &runtime).unwrap_err();
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RejectDeleteWithLiveRelations,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

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

    let error = txn.commit(execution, &runtime).unwrap_err();
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, _relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn.commit(execution, &runtime).unwrap_err();
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationDeletionInSameCommit,
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let (source, _target, relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    let deleted = delete_entity(execution, &runtime, source);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted.snapshot, execution)
        .expect("read request remains live")
        .unwrap();

    assert!(read.get_relation(relation).is_none());
}

#[test]
fn relation_integrity_commit_boundary_allows_relation_retirement_when_policy_retains_for_audit() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    let deleted = delete_entity(execution, &runtime, source);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let relation = read.get_relation(relation).unwrap();

    assert_eq!(
        relation.lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}

#[test]
fn historical_relation_lifecycle_does_not_reveal_future_audit_retention_after_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, _target, relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "historical-lifecycle");
    let live_version = runtime.current_version_id();
    let retired = delete_entity(execution, &runtime, source);
    create_entity(execution, &runtime, "after-audit-retirement");
    let post_retirement_version = runtime.current_version_id();

    assert_relation_lifecycle_at_version(
        execution,
        &runtime,
        relation,
        live_version,
        RecordLifecycleState::Live,
        None,
    );
    assert_relation_lifecycle_at_version(
        execution,
        &runtime,
        relation,
        retired.version_id,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
    assert_relation_lifecycle_at_version(
        execution,
        &runtime,
        relation,
        post_retirement_version,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );

    let (_, recovered) = checkpoint_and_recover_with(execution, &runtime, || {
        endpoint_deletion_runtime(
            crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
            CascadeDeletePolicy::RetainDanglingForAudit,
        )
    });
    assert_relation_lifecycle_at_version(
        execution,
        &recovered,
        relation,
        live_version,
        RecordLifecycleState::Live,
        None,
    );
    assert_relation_lifecycle_at_version(
        execution,
        &recovered,
        relation,
        retired.version_id,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
    assert_relation_lifecycle_at_version(
        execution,
        &recovered,
        relation,
        post_retirement_version,
        RecordLifecycleState::RetainedDanglingForAudit,
        Some(retired.version_id),
    );
}

fn assert_relation_lifecycle_at_version(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    relation: crate::identity::data::RelationId,
    version: crate::identity::data::VersionId,
    lifecycle: RecordLifecycleState,
    retired_at: Option<crate::identity::data::VersionId>,
) {
    let view = runtime
        .read_truth()
        .read_version(execution, version)
        .expect("read request remains live");
    let record = view
        .get_relation(relation)
        .expect("audit-retained relation remains materialized");
    assert_eq!(record.lifecycle, lifecycle);
    assert_eq!(record.retired_at_version, retired_at);
}

#[test]
fn relation_integrity_commit_boundary_rejects_relation_retirement_under_cascade_policy() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::CascadeDeleteRelations,
    );
    let (source, _target, _relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    let mut txn = crate::tests::support::test_owner_begin_transaction_for_main(&runtime);
    txn.push_batch(
        WorkerIntentBatch::new("delete-source").push(MutationIntent::Entity(
            EntityMutationIntent::Delete(DeleteEntityIntent { entity_id: source }),
        )),
        Allocation::SystemAllocation,
    )
    .expect("test staging stays within configured resource budgets");

    let error = txn.commit(execution, &runtime).unwrap_err();
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
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = endpoint_deletion_runtime(
        crate::schema::data::EndpointDeletionIntegrityMode::RequireRelationRetirement,
        CascadeDeletePolicy::RetainDanglingForAudit,
    );
    let (source, target, relation) =
        create_endpoint_deletion_relation_fixture(execution, &runtime, "live");

    delete_entity(execution, &runtime, source);
    let deleted_target = delete_entity(execution, &runtime, target);
    let read = runtime
        .read_truth()
        .read_snapshot(&deleted_target.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let relation = read.get_relation(relation).unwrap();

    assert_eq!(
        relation.lifecycle,
        RecordLifecycleState::RetainedDanglingForAudit
    );
}

#[path = "endpoint_deletion/branch_local_history.rs"]
mod branch_local_history;
