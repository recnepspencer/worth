use crate::publication::cdc::data::SubscriberContractDeclaration;
use crate::tests::support::*;

#[test]
fn subscriber_stream_rejects_schema_unsupported_checkpoint() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let mismatched_checkpoint =
        checkpoint_for_schema_version(PatchStreamPosition(1), SchemaVersionId(99));
    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(
            mismatched_checkpoint,
            1,
        ))
        .unwrap_err();

    assert_eq!(error.class, SubscriberStreamFailureClass::SchemaUnsupported);
}

#[test]
fn subscriber_stream_rejects_checkpoint_without_history_or_durable_coverage() {
    let runtime = persisted_runtime_with_test_schema();
    let missing_checkpoint =
        checkpoint_for_schema_version(PatchStreamPosition(42), SchemaVersionId(1));
    let error = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::resume_after(missing_checkpoint, 1))
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::DurableCoverageGap
    );
    assert!(error
        .diagnostics
        .iter()
        .all(|artifact| artifact.scope == DiagnosticsScope::Replay));
}

#[test]
fn subscriber_stream_rejects_checkpoint_with_mismatched_contract_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _ = create_entity_outcome(execution, &runtime, "anchor");

    let batch = runtime
        .publication()
        .read_subscriber_stream(SubscriberResumeRequest::from_head(1))
        .unwrap();
    let checkpoint = batch.next_checkpoint.unwrap();
    let requested_contract = SubscriberContractDeclaration {
        contract_id: "subscriber.contract.geometry.v2".to_string(),
        ..SubscriberContractDeclaration::default()
    };

    let error = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::resume_after(checkpoint, 1)
                .with_subscriber_contract(requested_contract),
        )
        .unwrap_err();

    assert_eq!(
        error.class,
        SubscriberStreamFailureClass::SubscriberContractMismatch
    );
}
