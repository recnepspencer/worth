//! Authored body commitments observed through Native's public owner.
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{AspectKey, AspectValue, FieldKey};
use worth_relational::facade::{
    identity::*, mvcc::*, runtime::*, schema::*, symbols::*, transactions::*,
};
use worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request;
fn batch(value: i64) -> WorkerIntentBatch {
    WorkerIntentBatch::new("authored").push(MutationIntent::Create(CreateIntent::Entity(
        EntitySpec {
            partition_id: PartitionId(1),
            kind_id: KindId(2),
            client_key: ClientKey::raw("first"),
            fields: AspectFieldPatch::from_locator(
                planned_single_field_locator(
                    AspectKey::new("test.body").unwrap(),
                    FieldKey::new("value").unwrap(),
                ),
                AspectValue::Int64(value),
            ),
        },
    )))
}

#[test]
fn authored_body_commitment_covers_values_order_and_batch_posture() {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let intent = RelationalTransactionIntent::ordinary();
    let hash = |batches: &[WorkerIntentBatch]| {
        home_program_transition_body_digest(&execution, &intent, batches).unwrap()
    };
    assert_eq!(hash(&[batch(1)]), hash(&[batch(1)]));
    assert_ne!(hash(&[batch(1)]), hash(&[batch(2)]));
    assert_ne!(hash(&[batch(1), batch(2)]), hash(&[batch(2), batch(1)]));
    let mut changed = batch(1);
    changed.worker_local_only = false;
    assert_ne!(hash(&[batch(1)]), hash(&[changed]));
    assert_ne!(
        hash(&[batch(1)]),
        hash(&[batch(1).with_partition_key("other")])
    );
}

#[test]
fn body_fingerprinting_refuses_the_original_cancelled_request() {
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(0)),
        cancellation.token(),
        None,
    );
    let execution = ExecutionRequest::serial(&request);
    assert!(matches!(
        home_program_transition_body_digest(
            &execution,
            &RelationalTransactionIntent::ordinary(),
            &[batch(1)]
        ),
        Err(RelationalHomeProgramBodyDenial::Request(_))
    ));
}
#[test]
fn authored_body_commitment_covers_schema_policy() {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let proposal = ProposedSchemaTransition {
        source_schema_id: SchemaId("source".into()),
        source_schema_version_id: SchemaVersionId(1),
        target_schema_id: SchemaId("target".into()),
        target_schema_version_id: SchemaVersionId(2),
        diff_atoms: vec![],
    };
    let one = RelationalTransactionIntent::ordinary().with_schema_transition(
        proposal.clone(),
        Some(SchemaReconciliationPolicy::PreserveInformation),
    );
    let other = RelationalTransactionIntent::ordinary().with_schema_transition(
        proposal,
        Some(SchemaReconciliationPolicy::PreserveTargetContract),
    );
    assert_ne!(
        home_program_transition_body_digest(&execution, &one, &[]).unwrap(),
        home_program_transition_body_digest(&execution, &other, &[]).unwrap()
    );
    assert_ne!(
        home_program_transition_body_digest(&execution, &one, &[]).unwrap(),
        home_program_transition_body_digest(
            &execution,
            &RelationalTransactionIntent::ordinary(),
            &[]
        )
        .unwrap()
    );
}
