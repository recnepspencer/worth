//! Real cancelled native reads retain control causes through their Query callers.
use crate::process::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::stops::leased_requests::*;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind as AttemptKind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Control;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionDenialKind as SessionKind;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionFailure;
use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionProtocolStage;
use worth_execution::CancellationSource;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_execution::ExecutionRequest;
use worth_execution::SerialMemoryBudget;
use worth_execution::SerialRequest;

pub fn serial(token: CancellationToken) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(None), token, None)
}

#[test]
fn cancelled_native_preparation_is_a_control_stop_not_no_kind() {
    let runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build()
        .expect("valid native runtime configuration");
    let source = CancellationSource::new();
    let request = serial(source.token());
    let execution = ExecutionRequest::serial(&request);
    let admitted = transaction(&runtime, "admitted")
        .commit(&execution, &runtime)
        .unwrap();
    assert_eq!(admitted.changed_records.len(), 1);
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &admitted.snapshot,
    );
    let pending = transaction(&runtime, "cancelled");
    source.cancel();
    let error = pending.commit(&execution, &runtime).unwrap_err();
    let failure = WorthQueryProviderSessionFailure::new(
        SessionKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        "native preparation refused",
        Default::default(),
    )
    .with_native_preparation_error_for_test(error);
    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::relational_execution_denial::native_preparation_kind(&failure), Err(Control::Cancelled));
    let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(failure);
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(Control::Cancelled))
    );
}

#[test]
fn cancelled_relation_equality_is_a_control_stop_not_false() {
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_query_execution::facade::domain_computation::WorthQueryDecisionReadSetDenialKind as ReadSet;
    use worth_relational::facade::transactions::CommitExecutionDenialKind;
    use worth_relational::facade::transactions::EntityReference;
    use worth_relational::facade::transactions::RecordRef;
    use worth_relational::facade::transactions::RelationSpec;
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build()
        .expect("valid native runtime configuration");
    let source = CancellationSource::new();
    let request = serial(source.token());
    let execution = ExecutionRequest::serial(&request);
    let create_entity = |key| {
        let committed = transaction(&runtime, key)
            .commit(&execution, &runtime)
            .unwrap();
        let RecordRef::Entity(entity) = committed.changed_records[0] else {
            panic!("entity commit produced its entity");
        };
        worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
            &mut runtime,
            &committed.snapshot,
        );
        entity
    };
    let from = create_entity("from");
    let to = create_entity("to");
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut pending = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    pending
        .push_batch(
            WorkerIntentBatch::new("relation").push(MutationIntent::Create(
                CreateIntent::Relation(RelationSpec {
                    partition_id: PartitionId::main(),
                    kind_id: KindId(2),
                    client_key: ClientKey::raw("relation"),
                    source: EntityReference::Existing(from),
                    target: EntityReference::Existing(to),
                    fields: AspectFieldPatch::default(),
                }),
            )),
            Allocation::SystemAllocation,
        )
        .unwrap();
    let committed = pending.commit(&execution, &runtime).unwrap();
    let RecordRef::Relation(relation) = committed.changed_records[0] else {
        panic!("relation commit produced its relation");
    };
    let fact = WorthQueryApplicationObservedFact::Relation {
        relation_kind: KindId(2),
        from,
        to,
        matching_relations: vec![relation],
    };
    assert!(fact
        .remains_equal_in_for_test(&runtime, &execution, &committed.snapshot)
        .unwrap());
    source.cancel();
    let denial = fact
        .remains_equal_in_for_test(&runtime, &execution, &committed.snapshot)
        .unwrap_err();
    assert_eq!(denial.kind(), AttemptKind::Cancelled);
    assert_eq!(
        denial
            .native_query_read_stop()
            .unwrap()
            .execution_denial()
            .unwrap()
            .kind,
        CommitExecutionDenialKind::Cause(Cause::Cancelled)
    );
    let failure =
        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::decision_facts::fact_read_set_failure(
            denial,
        );
    assert_eq!(
        failure.kind(),
        ReadSet::RequestInterrupted(WorthQueryRequestInterruption::Cancelled)
    );
    assert_eq!(
        failure
            .native_query_read_stop()
            .unwrap()
            .execution_denial()
            .unwrap()
            .kind,
        CommitExecutionDenialKind::Cause(Cause::Cancelled)
    );
    let denial = worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenial::decision_read_set_denied(failure);
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(Control::Cancelled))
    );
    worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(
        &mut runtime,
        &committed.snapshot,
    );
}
