use super::*;

#[test]
fn noncanonical_or_oversized_wire_is_denied() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let fixture = authorization_fixture(execution);
    let wire = stamp_now(execution, &fixture).to_wire(execution).unwrap();
    assert_eq!(
        RelationalAuthorizationDurableDependencies::from_wire(execution, &wire.to_uppercase())
            .unwrap_err(),
        RelationalAuthorizationDependencyDenial::MalformedWire
    );
    assert_eq!(
        RelationalAuthorizationDurableDependencies::from_wire(
            execution,
            &"0".repeat(MAXIMUM_AUTHORIZATION_DEPENDENCY_BYTES * 2 + 2)
        )
        .unwrap_err(),
        RelationalAuthorizationDependencyDenial::ByteBudgetExceeded
    );
}
