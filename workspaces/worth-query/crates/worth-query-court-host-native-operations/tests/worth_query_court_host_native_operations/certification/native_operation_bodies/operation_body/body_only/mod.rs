pub(crate) use worth_query_testkit_certification_operation_body::facade::certification::native_operation_bodies::operation_body::body_only::*;

#[test]
fn authored_body_without_a_mutation_binding_is_denied_before_initialization() {
    let result = in_memory(
        BodyOnlySchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host_limits(),
        |_phase, _graph, _schema| panic!("missing body execution must deny before initial state"),
    );
    match result {
        Err(WorthQueryInMemoryApplicationDenial::Contributions(denial)) => assert_eq!(
            denial.kind(),
            WorthQueryPrimaryGraphInstallationDenialKind::OperationBodyUnavailable,
        ),
        Err(other) => panic!("expected missing body coverage: {other:?}"),
        Ok(_) => panic!("a metadata-only operation body cannot be installed"),
    }
}
