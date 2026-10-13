use crate::tests::support::*;

#[test]
fn read_records_expose_visibility_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let outcome = create_entity_outcome(execution, &runtime, "visible");
    let read = runtime
        .read_truth()
        .read_snapshot(&outcome.snapshot, execution)
        .expect("read request remains live")
        .unwrap();
    let record = read.entities().first().unwrap();

    assert_eq!(record.created_at_version, outcome.version_id);
    assert_eq!(record.retired_at_version, None);
}
