use crate::publication::cdc::data::SubscriberContractDeclaration;
use crate::schema::data::DescriptorSemanticsVersion;
use crate::tests::support::*;

#[test]
fn subscriber_stream_propagates_declared_contract_identity_into_checkpoints() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let _first = create_entity_outcome(execution, &runtime, "a");
    let _second = create_entity_outcome(execution, &runtime, "b");
    let contract = SubscriberContractDeclaration {
        contract_id: "subscriber.contract.geometry.v1".to_string(),
        ..SubscriberContractDeclaration::default()
    };

    let batch = runtime
        .publication()
        .read_subscriber_stream(
            SubscriberResumeRequest::from_head(2).with_subscriber_contract(contract.clone()),
        )
        .unwrap();

    let next_checkpoint = batch.next_checkpoint.unwrap();
    let latest_available_checkpoint = batch.latest_available_checkpoint.unwrap();

    assert_eq!(
        next_checkpoint.subscriber_contract_id(),
        contract.contract_id.as_str()
    );
    assert_eq!(
        latest_available_checkpoint.subscriber_contract_id(),
        contract.contract_id.as_str()
    );
    assert_eq!(
        next_checkpoint.descriptor_semantics_version(),
        DescriptorSemanticsVersion::default()
    );
    assert_eq!(
        next_checkpoint
            .normalized_continuation_proof()
            .descriptor_semantics_version(),
        DescriptorSemanticsVersion::default()
    );
}
