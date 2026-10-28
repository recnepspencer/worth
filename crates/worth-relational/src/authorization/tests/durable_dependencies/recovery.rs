use super::*;

#[test]
fn source_native_stamp_compares_after_checkpoint_recovery() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime_with_test_schema();
    let principal = create_entity(execution, &runtime, "principal");
    let role = create_entity(execution, &runtime, "approver");
    let scope = create_entity(execution, &runtime, "payment");
    create_relation(execution, &runtime, principal, role, "principal-role");
    create_relation(execution, &runtime, role, scope, "role-scope");
    let before_evidence = runtime
        .observe_authorization(
            allow_plan(
                runtime
                    .visibility_authority()
                    .snapshot()
                    .expect("current snapshot fixture is admitted"),
                principal,
                scope,
                [],
            ),
            execution,
        )
        .unwrap();
    let wire = runtime
        .capture_authorization_durable_dependencies(&before_evidence, execution)
        .unwrap()
        .to_wire(execution)
        .unwrap();
    crate::tests::support::checkpoint_durably(execution, &runtime).unwrap();
    let recovery = crate::tests::support::recovery_plan_for(
        execution,
        &runtime,
        RecoveryVerificationMode::NormalRecoveryVerification,
    )
    .unwrap();
    let mut recovered = runtime_with_test_schema();
    recovered
        .durability_recovery()
        .recover(execution, recovery)
        .unwrap();
    let fresh_evidence = recovered
        .observe_authorization(
            allow_plan(
                recovered
                    .visibility_authority()
                    .snapshot()
                    .expect("current snapshot fixture is admitted"),
                principal,
                scope,
                [],
            ),
            execution,
        )
        .unwrap();
    let fresh = recovered
        .capture_authorization_durable_dependencies(&fresh_evidence, execution)
        .unwrap();
    assert!(
        RelationalAuthorizationDurableDependencies::from_wire(execution, &wire)
            .unwrap()
            .matches(&fresh)
    );
    assert!(
        RelationalAuthorizationDurableDependencies::from_wire(execution, &wire)
            .unwrap()
            .source_revisions_match(
                &recovered,
                execution,
                &recovered
                    .visibility_authority()
                    .snapshot()
                    .expect("current snapshot fixture is admitted")
            )
            .unwrap()
    );
}
