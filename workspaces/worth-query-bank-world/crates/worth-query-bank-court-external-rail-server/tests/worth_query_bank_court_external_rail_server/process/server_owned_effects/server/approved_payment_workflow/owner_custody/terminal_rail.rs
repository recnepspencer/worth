pub(crate) use worth_query_bank_testkit_server::facade::approved_payment_workflow::owner_custody::terminal_rail::*;

#[test]
fn native_owner_recovery_preserves_the_completed_external_settlement() {
    let fixture = fixture::ordinary_read_world_with_approval_authentication(
        "terminal-external-settlement",
        super::super::authentication::approval_configuration(),
    );
    let ready = ReadyPaymentWorld::from_fixture(fixture, FaultScript::CommitThenLoseResponse);
    let route = ready.install_synchronous_completion();
    let before_payment = support::balances(&ready);
    let runtime = &ready.fixture.world.runtime;
    let request = runtime
        .request(&ready.principal, &ready.scope)
        .on_branch(ready.instance.branch());
    let operation_key = key("approved-payment:operation:perform");
    let operation = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .retain_apply(
            &ready.operation,
            ready.authority.clone(),
            operation_key.clone(),
        )
        .unwrap();
    let mut operation = match operation.advance(&request) {
        WorthQueryWorkflowOperationAdvance::Progress(operation) => operation,
        _ => panic!("the lost real rail response must retain the original owner"),
    };
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let token = ready.rail.attempts()[0].token().try_into().unwrap();
    assert!(runtime
        .observe_payment_rail_completion(&route, token)
        .is_none());

    // Inspect the actual same-key receipt while retaining the original operation owner.
    let source = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .advanced()
        .execute_operation(
            ready.instance.clone(),
            &ready.operation,
            ready.authority.clone(),
            &operation_key,
        )
        .unwrap();
    let bank_server::BankApprovedPaymentApplyOutcome::RecoveredCommitted(source) = source else {
        panic!("same-key inspection must return original data without a performed owner");
    };
    assert_eq!(
        ready.rail.attempts().len(),
        1,
        "receipt observation cannot redispatch"
    );
    let original_world = {
        let inspection = operation
            .inspect_source(&request)
            .expect("freshly admitted actual owner inspection");
        let original = inspection
            .source()
            .live_receipt()
            .expect("the performing operation retains its genuine first receipt");
        worth_query_bank_testkit_server::facade::recorded_commit_axes::assert_agrees(
            &source, original,
        );
        original.correction_source().world().clone()
    };
    let after_payment = support::balances(&ready);
    assert_ne!(after_payment, before_payment);

    ready
        .rail
        .under(FaultScript::Succeed, Duration::from_millis(150));
    let completed = loop {
        match operation.advance(&request) {
            WorthQueryWorkflowOperationAdvance::Progress(next) => operation = next,
            WorthQueryWorkflowOperationAdvance::Completed(completed) => break completed,
            WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("original real rail recovery refused: {denial:?}");
            }
            WorthQueryWorkflowOperationAdvance::Terminal(terminal) => {
                let _original_owner = terminal;
                panic!("original real rail recovery terminated");
            }
        }
    };
    let terminal = runtime
        .observe_payment_rail_completion(&route, token)
        .unwrap();
    assert_eq!(terminal.original_world_commit(), &original_world);
    assert_eq!(support::balances(&ready), after_payment);
    let accepted = runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .accept_owned_applied(
            ready.instance.clone(),
            ready.authority.clone(),
            completed,
            &key("approved-payment:terminal-settlement:accept-owner"),
        )
        .unwrap_or_else(|(_owner, cause)| {
            panic!("authentic completed owner settlement: {cause:?}")
        });
    require_completed(accepted.progress, "apply");
    accepted
        .operation
        .close(&request)
        .unwrap_or_else(|_| panic!("original owner closes normally"));
    for (step, operation_key) in [
        (
            "await-inbound",
            "approved-payment:terminal-settlement:consume-inbound",
        ),
        ("completed", "approved-payment:terminal-settlement:end"),
    ] {
        require_completed(
            runtime
                .approved_business_payment(&ready.principal, &ready.scope)
                .advance(
                    ready.instance.clone(),
                    ready.authority.clone(),
                    &key(operation_key),
                )
                .unwrap(),
            step,
        );
    }

    assert_eq!(support::balances(&ready), after_payment);
    let retained = runtime
        .observe_payment_rail_completion(&route, token)
        .unwrap();
    assert_eq!(
        retained.original_world_commit(),
        terminal.original_world_commit()
    );
    assert_eq!(
        retained.completion_world_commit(),
        terminal.completion_world_commit()
    );
    assert_eq!(retained.completion_attempt(), terminal.completion_attempt());
    assert_eq!(ready.rail.attempts().len(), 2);
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let payment = request
        .query(bank_domain::queries::payment(ready.fixture.payment))
        .execute()
        .unwrap();
    assert_eq!(payment.rows()[0].status(), PaymentStatus::Committed);
    assert_eq!(
        payment.rows()[0].deciding_principal(),
        Some(fixture::principal_id(fixture::APPROVER))
    );
    let cancelled = super::cancel(
        &ready,
        "approved-payment:terminal-settlement:cancel-completed",
    );
    assert!(matches!(cancelled,
        Err(bank_server::BankApprovedPaymentWorkflowError::InstanceCancellation(
            WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(WorkflowInstancePreparationDenial::Attempt(ref denial))))
                if denial.kind() == WorthQueryApplicationAttemptDenialKind::WorkflowInstanceCompleted));
}
