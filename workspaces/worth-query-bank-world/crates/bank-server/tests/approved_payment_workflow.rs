#[path = "approved_payment_workflow/actor_handoff.rs"]
mod actor_handoff;
#[path = "approved_payment_workflow/approval.rs"]
mod approval;
#[path = "approved_payment_workflow/assertions.rs"]
mod assertions;
#[path = "approved_payment_workflow/authentication.rs"]
mod authentication;
#[allow(
    dead_code,
    reason = "the shared read fixture has discovery-only helpers exercised by its owning test binary"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[path = "approved_payment_workflow/journey.rs"]
mod journey;
#[path = "approved_payment_workflow/postures.rs"]
mod postures;
#[allow(
    dead_code,
    reason = "the shared rail adapter exposes additional custody probes used by its owning court"
)]
#[path = "ordinary_mutations/estate_operations/external_effect_dispatch/rail_transport.rs"]
mod rail_transport;
#[path = "approved_payment_workflow/rejection.rs"]
mod rejection;
mod support;

use std::{sync::Arc, time::Duration};

use bank_domain::{
    queries::payment,
    schema::{ApprovePayment, PaymentStatus},
};
use bank_external_rail::{test_control::FaultScript, LedgerStatus};
use fixture::{ordinary_read_world_with_approval_authentication, principal_id, APPROVER};
use support::request_scope;
use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorthQueryOrdinaryWorkflowRunStop,
    WorthQueryWorkflowOperationOwnerAcceptanceDenial, WorthQueryWorkflowOperationOwnerPosture,
    WorthQueryWorkflowOperationRecoveryPreparationDenial,
};
use worth_query_host::facade::primary_graph::WorthQueryExternalDispatchPostureKind;

use assertions::{key, require_completed};
use rail_transport::{spawn_rail, BankEstateRailTransport};

#[test]
fn approved_business_payment_runs_through_query_and_commits_the_real_payment_operation() {
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
        .perform_apply(
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
        .perform_apply(
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
        .accept_applied(
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
        bank_server::BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(
            WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            )
        )
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
        .prepare_apply_recovery(
            &still_required,
            authority.clone(),
            &key("approved-payment:operation:foreign-recovery"),
        )
        .err()
        .expect("a different workflow operation command cannot claim recovery custody");
    assert!(matches!(
        mismatched_recovery,
        bank_server::BankApprovedPaymentWorkflowError::OperationRecoveryPreparation(
            WorthQueryWorkflowOperationRecoveryPreparationDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::IntentDrift
            )
        )
    ));
    let recovery = workflow
        .prepare_apply_recovery(
            &still_required,
            authority.clone(),
            &key("approved-payment:operation:perform"),
        )
        .expect("the exact workflow-bound operation prepares recovery")
        .safe_retry()
        .expect("fresh recovery authority re-dispatches the committed outbox");
    require_completed(
        workflow
            .accept_recovered_applied(
                instance.clone(),
                &still_required,
                authority.clone(),
                authority.clone(),
                &key("approved-payment:operation:perform"),
                &recovery,
                &key("approved-payment:operation:accept:recovered"),
            )
            .expect("the exact completed recovery proof settles the operation"),
        "apply",
    );
    require_completed(
        workflow
            .accept_recovered_applied(
                instance.clone(),
                &operation,
                authority.clone(),
                authority.clone(),
                &key("approved-payment:operation:perform"),
                &recovery,
                &key("approved-payment:operation:accept:recovered"),
            )
            .expect("the recovered workflow settlement replays without another dispatch"),
        "apply",
    );
    assert_eq!(settlement_rail.attempts().len(), 2);
    assert_eq!(settlement_rail.completed_effect_count(), 1);
    let keys = [key("approved-payment:advance:completed")];
    let completed = workflow.run(instance.clone(), authority, &keys);
    assert_eq!(completed.attempted_steps(), 1);
    assert_eq!(completed.transitions().len(), 1);
    assert_eq!(completed.transitions()[0].node_path(), "completed");
    assert!(matches!(
        completed.stop(),
        WorthQueryOrdinaryWorkflowRunStop::Terminal
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
