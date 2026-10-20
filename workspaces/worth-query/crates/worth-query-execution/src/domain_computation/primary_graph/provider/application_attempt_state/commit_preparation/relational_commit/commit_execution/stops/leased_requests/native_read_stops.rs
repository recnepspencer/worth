//! Real cancelled native reads retain control causes through their Query callers.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact,
    WorthQueryApplicationAttemptDenialKind as AttemptKind,
};
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Control,
    WorthQueryProviderSessionDenialKind as SessionKind, WorthQueryProviderSessionFailure,
    WorthQueryProviderSessionProtocolStage,
};
use worth_execution::CancellationSource;
use worth_execution::{
    ExecutionAllocationPolicy as Allocation, ExecutionRequest, SerialMemoryBudget, SerialRequest,
};

fn serial(token: CancellationToken) -> SerialRequest {
    SerialRequest::from_memory(SerialMemoryBudget::new(None), token, None)
}

#[test]
fn cancelled_native_preparation_is_a_control_stop_not_no_kind() {
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build();
    let source = CancellationSource::new();
    let request = serial(source.token());
    let execution = ExecutionRequest::serial(&request);
    let admitted = transaction(&runtime, "admitted")
        .commit(&execution, &runtime)
        .unwrap();
    assert_eq!(admitted.changed_records.len(), 1);
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &admitted.snapshot);
    let pending = transaction(&runtime, "cancelled");
    source.cancel();
    let error = pending.commit(&execution, &runtime).unwrap_err();
    let failure = WorthQueryProviderSessionFailure::new(
        SessionKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        "native preparation refused",
        Default::default(),
    )
    .with_native_preparation_error(error);
    assert_eq!(crate::domain_computation::primary_graph::provider::relational_execution_denial::native_preparation_kind(&failure), Err(Control::Cancelled));
    let denial = crate::domain_computation::primary_graph::application_attempt::provider_compare_denial::provider_session_denied(failure);
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(Control::Cancelled))
    );
}

#[test]
fn cancelled_relation_equality_is_a_control_stop_not_false() {
    use crate::domain_computation::WorthQueryDecisionReadSetDenialKind as ReadSet;
    use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, EntityReference, RecordRef, RelationSpec,
        RelationalExecutionDenialCause as Cause,
    };
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema())
        .build();
    let source = CancellationSource::new();
    let request = serial(source.token());
    let execution = ExecutionRequest::serial(&request);
    let mut create_entity = |key| {
        let committed = transaction(&runtime, key)
            .commit(&execution, &runtime)
            .unwrap();
        let RecordRef::Entity(entity) = committed.changed_records[0] else {
            panic!("entity commit produced its entity");
        };
        crate::relational_snapshot_release::release_query_snapshot(
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
        .remains_equal_in(&runtime, &execution, &committed.snapshot)
        .unwrap());
    source.cancel();
    let denial = fact
        .remains_equal_in(&runtime, &execution, &committed.snapshot)
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
        crate::domain_computation::primary_graph::provider::decision_facts::fact_read_set_failure(
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
    let denial = crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenial::decision_read_set_denied(failure);
    assert_eq!(
        denial.execution_denial_cause(),
        Some(Err(Control::Cancelled))
    );
    crate::relational_snapshot_release::release_query_snapshot(&mut runtime, &committed.snapshot);
}
