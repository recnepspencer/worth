use super::fixtures::*;
use crate::tests::support::*;

#[test]
fn ordinary_author_cannot_rewrite_a_valid_platform_identity_record() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 0]));
    let rows = records(execution, &runtime, &born.snapshot);
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("hostile-private-write").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: rows[0].0,
                    fields: AspectFieldPatch::new(rows[0].1.fields()),
                }),
            )),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let before = runtime.history().branch_head(&BranchId("main".into()));
    assert!(matches!(transaction.commit(execution, &runtime),
        Err(TransactionCommitError::Conflict { error, .. })
            if matches!(error.class, crate::transactions::data::ConflictClass::KindSchemaMismatch { .. })));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert_eq!(records(execution, &runtime, &born.snapshot), rows);
    let ids = members(&born, 2);
    let lawful = commit(
        execution,
        &runtime,
        "main",
        rewire(execution, &runtime, &born.snapshot, &[(ids[0], ids[1])]),
    );
    assert_eq!(records(execution, &runtime, &lawful.snapshot), rows);
}

#[test]
fn added_private_rows_must_pass_final_snapshot_admission_after_prevalidation() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::snapshot_publication_blocking(
                InvariantRule::MaxSnapshotEntities(4),
            )],
        })
        .build();
    let born = commit(execution, &runtime, "main", birth(&[1, 2, 0]));
    let ids = members(&born, 3);
    let before = runtime.history().branch_head(&BranchId("main".into()));
    let rows = records(execution, &runtime, &born.snapshot);
    let batch = rewire(
        execution,
        &runtime,
        &born.snapshot,
        &[(ids[0], ids[0]), (ids[2], ids[1])],
    );
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            batch,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(
        transaction.commit(execution, &runtime),
        Err(TransactionCommitError::Publication { .. })
    ));
    assert_eq!(
        runtime.history().branch_head(&BranchId("main".into())),
        before
    );
    assert_eq!(records(execution, &runtime, &born.snapshot), rows);
    assert!(runtime.record_identity.pending_snapshot().is_empty());
}

#[test]
fn preparation_discard_does_not_publish_generated_metadata_or_reservations() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let runtime = runtime();
    let before = runtime.history().immutable_commit_count();
    let mut transaction = test_owner_begin_transaction_for_main(&runtime);
    transaction
        .push_batch(
            birth(&[1, 0]),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let candidate = runtime
        .prepare_branch_transaction(execution, transaction)
        .unwrap();
    assert!(
        candidate.reservation_count() > 4,
        "aggregate row and memberships share the candidate allocation stream"
    );
    assert_eq!(runtime.history().immutable_commit_count(), before);
    runtime.discard_prepared_candidate(candidate).unwrap();
    assert!(runtime.record_identity.pending_snapshot().is_empty());
    assert_eq!(runtime.history().immutable_commit_count(), before);
}

#[test]
fn native_unaccounted_serial_request_does_not_invent_accounting() {
    use worth_foundational::{DeterminismContract, ExecutionPosture, ExecutionRequestPolicy};
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::new(1).unwrap(),
    );
    let host = worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    assert!(!execution.has_accounting());
    let runtime = runtime();
    let born = commit(execution, &runtime, "main", birth(&[1, 0]));
    let inspection_host = crate::test_execution_policy::serial_request();
    let inspection = worth_execution::ExecutionRequest::serial(&inspection_host);
    assert_eq!(records(&inspection, &runtime, &born.snapshot).len(), 1);
}

#[test]
fn metadata_registration_refuses_conflicting_policies_and_accepts_its_exact_twin() {
    let mut base = registry();
    let original = base.relation_kinds.remove(&KindId(5)).unwrap();
    assert!(base
        .clone()
        .register_relation_kind(original.clone())
        .is_ok());
    let mut hostile = original.clone();
    hostile.relation_integrity.cardinality_contracts[0].source_max = Some(0);
    assert!(base.clone().register_relation_kind(hostile).is_err());
    let mut hostile = original.clone();
    hostile.cross_context_policy = CrossContextPolicy::SchemaControlled;
    assert!(base.clone().register_relation_kind(hostile).is_err());
    let mut hostile = original;
    hostile.cascade_delete_policy = CascadeDeletePolicy::CascadeDeleteRelations;
    assert!(base.register_relation_kind(hostile).is_err());
}

#[test]
fn conflicting_lineage_law_is_refused_at_descriptor_binding() {
    let mut base = registry();
    let membership = base.relation_kinds.remove(&KindId(5)).unwrap();
    let mut lineage = base.relation_kinds.remove(&KindId(4)).unwrap();
    lineage.relation_integrity.cardinality_contracts.push(
        crate::schema::data::CardinalityContractDeclaration {
            contract_id: "hostile-lineage-max-zero".into(),
            source_min: None,
            source_max: Some(0),
            target_min: None,
            target_max: None,
            pair_min: None,
            pair_max: None,
            pair_min_semantics: crate::schema::data::PairMinimumSemantics::ObservedDirectedPairs,
            minimum_enforcement: crate::schema::data::MinimumCardinalityEnforcement::CommitBoundary,
        },
    );
    let base = base.register_relation_kind(lineage).unwrap();
    assert!(base.register_relation_kind(membership).is_err());
}
