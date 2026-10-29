pub(crate) use worth_query_bank_worlds::facade::server::approved_payment_workflow::*;

pub(crate) mod actor_handoff;

pub(crate) mod operation_authorization;

pub(crate) mod owner_custody;

pub(crate) mod owning;

pub(crate) mod postures;

pub(crate) mod program_retirement;

pub(crate) mod rejection;

pub(crate) mod relabel;

#[test]
fn native_lost_reply_payment_recovers_owner_custody_and_real_rail_effect() {
    let fixture = ordinary_read_world_with_approval_authentication(
        "approved-payment-workflow",
        authentication::approval_configuration(),
    );
    let rail_process = spawn_rail();
    let settlement_rail = Arc::new(BankEstateRailTransport::connected_to(
        rail_process.local_addr(),
        rail_process.test_control_addr(),
    ));
    settlement_rail.under(FaultScript::CommitThenLoseResponse, Duration::from_secs(5));
    fixture
        .world
        .runtime
        .install_external_effect_transport(settlement_rail.clone())
        .expect("the approved-payment settlement rail installs");
    let approver = fixture.authenticate(APPROVER);
    let scope = request_scope();
    let workflow = fixture
        .world
        .runtime
        .approved_business_payment(&approver, &scope);
    let authority = ApprovePayment {
        payment: fixture.payment,
        approver: principal_id(APPROVER),
    };

    let (instance, operation) = journey::prepare_approved_payment_operation(&workflow, &authority);
    let performed = workflow
        .advanced()
        .execute_operation(
            instance.clone(),
            &operation,
            authority.clone(),
            &key("approved-payment:operation:perform"),
        )
        .expect("the admitted Bank approver reaches the real payment effect")
        .into_performed()
        .expect("the admitted Bank payment effect commits");
    assert!(performed.newly_committed());
    assert_eq!(performed.emitted_effect_count(), 3);
    assert!(performed.co_committed_dispatch_outbox());
    assert_eq!(
        performed.external_dispatch_posture(),
        Some(WorthQueryExternalDispatchPostureKind::Unresolved)
    );
    let replayed = workflow
        .advanced()
        .execute_operation(
            instance.clone(),
            &operation,
            authority.clone(),
            &key("approved-payment:operation:perform"),
        )
        .expect("the lost response retry reaches retained operation custody")
        .into_performed()
        .expect("the lost response retry recovers the performed payment operation");
    assert!(!replayed.newly_committed());
    let attempts = settlement_rail.attempts();
    assert_eq!(attempts.len(), 1);
    assert_eq!(settlement_rail.admission_count(), 1);
    assert_eq!(settlement_rail.completed_effect_count(), 1);
    assert_eq!(
        settlement_rail.ledger_status(&attempts[0]),
        LedgerStatus::Completed
    );
    drop(replayed);
    drop(performed);
    // Acceptance and recovery reconstruct custody from the owner after the
    // original effect response and its retry receipt have both been lost.
    let unresolved = workflow
        .advanced()
        .accept_native_effect(
            instance.clone(),
            &operation,
            authority.clone(),
            authority.clone(),
            &key("approved-payment:operation:perform"),
            &key("approved-payment:operation:accept:unresolved"),
        )
        .expect_err("unresolved external custody cannot settle the workflow operation");
    assert!(matches!(
        unresolved,
        bank_server::BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(denial) if matches!(denial.as_ref(), WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            ))
    ));
    let still_required = match workflow
        .advance(
            instance.clone(),
            authority.clone(),
            &key("approved-payment:advance:operation:recovery-pending"),
        )
        .expect("the unresolved receipt leaves the workflow awaiting its operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected the operation to remain pending, got {other:?}"),
    };
    assert_eq!(
        still_required.transition_identity(),
        operation.transition_identity()
    );
    let mismatched_recovery = workflow
        .advanced()
        .prepare_native_effect_recovery(
            &still_required,
            authority.clone(),
            &key("approved-payment:operation:foreign-recovery"),
        )
        .err()
        .expect("a different workflow operation command cannot claim recovery custody");
    assert!(matches!(
        mismatched_recovery,
        bank_server::BankApprovedPaymentWorkflowError::OperationRecoveryPreparation(denial) if matches!(denial.as_ref(), WorthQueryWorkflowOperationRecoveryPreparationDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::IntentDrift
            ))
    ));
    let prepared_recovery = workflow
        .advanced()
        .prepare_native_effect_recovery(
            &still_required,
            authority.clone(),
            &key("approved-payment:operation:perform"),
        )
        .expect("the exact workflow-bound operation prepares recovery");
    let recovery = fixture
        .world
        .runtime
        .with_application_advancement(&scope, |phase| prepared_recovery.safe_retry(&phase))
        .expect("fixture retry owns its host call")
        .expect("fresh recovery authority re-dispatches the committed outbox");
    let denied = workflow
        .advanced()
        .accept_native_recovered_effect(
            instance.clone(),
            &still_required,
            authority.clone(),
            authority.clone(),
            &key("approved-payment:operation:perform"),
            &recovery,
            &key("approved-payment:operation:accept:recovered"),
        )
        .expect_err("rail-local completion does not publish an authenticated inbound terminal");
    assert!(matches!(
        denied,
        bank_server::BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(_)
    ));
    assert_eq!(settlement_rail.attempts().len(), 2);
    assert_eq!(settlement_rail.completed_effect_count(), 1);
    let keys = [key("approved-payment:advance:without-inbound")];
    let pending = workflow.run(instance.clone(), authority, &keys);
    assert_eq!(pending.attempted_steps(), 1);
    assert!(pending.transitions().is_empty());
    assert!(matches!(
        pending.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Outcome(WorkflowProgressOutcome::AwaitingOperation(_))
    ));

    let observed = fixture
        .world
        .runtime
        .request(&approver, &scope)
        .on_branch(instance.branch())
        .query(payment(fixture.payment))
        .execute()
        .expect("the workflow branch exposes the committed payment");
    assert_eq!(observed.rows().len(), 1);
    assert_eq!(observed.rows()[0].status(), PaymentStatus::Committed);
    assert_eq!(
        observed.rows()[0].deciding_principal(),
        Some(principal_id(APPROVER))
    );
}
