#[allow(
    dead_code,
    reason = "the shared read fixture has discovery-only helpers exercised by its owning test binary"
)]
#[path = "ordinary_reads/fixture.rs"]
mod fixture;
#[allow(
    dead_code,
    reason = "the shared rail adapter exposes additional custody probes used by its owning court"
)]
#[path = "ordinary_mutations/estate_operations/external_effect_dispatch/rail_transport.rs"]
mod rail_transport;
mod support;

use std::{sync::Arc, time::Duration};

use bank_domain::{
    proposals::BankIdempotencyKey,
    queries::payment,
    schema::{ApprovePayment, PaymentStatus},
};
use bank_external_rail::{test_control::FaultScript, LedgerStatus};
use fixture::{ordinary_read_world, principal_id, APPROVER};
use support::request_scope;
use worth_query_host::facade::application_entry::{
    WorkflowApprovalDecision, WorkflowDefinitionExpectedPredecessor,
    WorkflowDefinitionPublicationOutcome, WorkflowInstanceStartOutcome, WorkflowProgressOutcome,
    WorkflowProposalOutcome,
};
use worth_query_host::facade::primary_graph::WorthQueryExternalDispatchPostureKind;

use rail_transport::{spawn_rail, BankEstateRailTransport};

#[test]
fn approved_business_payment_runs_through_query_and_commits_the_real_payment_operation() {
    let fixture = ordinary_read_world("approved-payment-workflow", 0);
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

    let published = match workflow
        .publish_definition(
            authority.clone(),
            WorkflowDefinitionExpectedPredecessor::Absent,
            &key("approved-payment:definition"),
        )
        .expect("the Bank-owned workflow definition publishes")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => published,
        other => panic!("expected definition publication, got {other:?}"),
    };
    let started = match workflow
        .start(
            published.definition().clone(),
            authority.clone(),
            &key("approved-payment:instance"),
        )
        .expect("the Bank starts its retained workflow")
    {
        WorkflowInstanceStartOutcome::Started(started) => started,
        other => panic!("expected workflow instance, got {other:?}"),
    };
    let proposal = match workflow
        .propose(
            started.instance().clone(),
            authority.clone(),
            &key("approved-payment:proposal"),
        )
        .expect("the typed payment proposal publishes")
    {
        WorkflowProposalOutcome::Published(proposal) => proposal,
        other => panic!("expected workflow proposal, got {other:?}"),
    };

    require_assessment(
        workflow
            .advance(
                started.instance().clone(),
                authority.clone(),
                &key("approved-payment:advance:payment"),
            )
            .expect("the proposal advances to payment assessment"),
        "review/payment",
    );
    let payment_assessment = workflow
        .settle_payment_assessment(
            started.instance().clone(),
            authority.clone(),
            &key("approved-payment:assessment:payment:settle"),
        )
        .expect("the payment assessment settles from PaymentDetailQuery");
    require_completed(
        workflow
            .accept_assessment(
                started.instance().clone(),
                authority.clone(),
                &payment_assessment,
                &key("approved-payment:assessment:payment:accept"),
            )
            .expect("the exact payment assessment is accepted"),
        "review/payment",
    );

    require_assessment(
        workflow
            .advance(
                started.instance().clone(),
                authority.clone(),
                &key("approved-payment:advance:independent"),
            )
            .expect("the payment review advances to independent assessment"),
        "review/independent",
    );
    let independent_assessment = workflow
        .settle_payment_assessment(
            started.instance().clone(),
            authority.clone(),
            &key("approved-payment:assessment:independent:settle"),
        )
        .expect("the independent assessment settles from the installed producer");
    require_completed(
        workflow
            .accept_assessment(
                started.instance().clone(),
                authority.clone(),
                &independent_assessment,
                &key("approved-payment:assessment:independent:accept"),
            )
            .expect("the exact independent assessment is accepted"),
        "review/independent",
    );
    require_completed(
        workflow
            .advance(
                started.instance().clone(),
                authority.clone(),
                &key("approved-payment:advance:evidence"),
            )
            .expect("both assessments satisfy the evidence join"),
        "review/evidence",
    );

    let approval = match workflow
        .advance(
            started.instance().clone(),
            authority.clone(),
            &key("approved-payment:advance:approval"),
        )
        .expect("the evidence join advances")
    {
        WorkflowProgressOutcome::AwaitingApproval(required) => required,
        other => panic!("expected approval requirement, got {other:?}"),
    };
    require_completed(
        workflow
            .approve(
                started.instance().clone(),
                &approval,
                proposal.proposal(),
                WorkflowApprovalDecision::Approve,
                authority.clone(),
                &key("approved-payment:approval"),
            )
            .expect("the Bank approver approves the exact proposal and evidence"),
        "approval",
    );

    let operation = match workflow
        .advance(
            started.instance().clone(),
            authority.clone(),
            &key("approved-payment:advance:operation"),
        )
        .expect("approval advances to the real payment operation")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected payment operation requirement, got {other:?}"),
    };
    let performed = workflow
        .perform_apply(
            started.instance().clone(),
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
            started.instance().clone(),
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
    let unresolved = workflow
        .accept_applied(
            started.instance().clone(),
            &operation,
            authority.clone(),
            &performed,
            &key("approved-payment:operation:accept:unresolved"),
        )
        .expect_err("unresolved external custody cannot settle the workflow operation");
    assert!(unresolved.to_string().contains("RecoveryRequired"));
    let still_required = match workflow
        .advance(
            started.instance().clone(),
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
            &operation,
            authority.clone(),
            &performed,
            &key("approved-payment:operation:foreign-recovery"),
        )
        .err()
        .expect("a different workflow operation command cannot claim recovery custody");
    assert!(mismatched_recovery
        .to_string()
        .contains("IdempotencyMismatch"));
    let recovery = workflow
        .prepare_apply_recovery(
            &operation,
            authority.clone(),
            &performed,
            &key("approved-payment:operation:perform"),
        )
        .expect("the exact workflow-bound operation prepares recovery")
        .safe_retry()
        .expect("fresh recovery authority re-dispatches the committed outbox");
    require_completed(
        workflow
            .accept_recovered_applied(
                started.instance().clone(),
                &operation,
                authority.clone(),
                &performed,
                &recovery,
                &key("approved-payment:operation:accept:recovered"),
            )
            .expect("the exact completed recovery proof settles the operation"),
        "apply",
    );
    require_completed(
        workflow
            .accept_recovered_applied(
                started.instance().clone(),
                &operation,
                authority.clone(),
                &performed,
                &recovery,
                &key("approved-payment:operation:accept:recovered"),
            )
            .expect("the recovered workflow settlement replays without another dispatch"),
        "apply",
    );
    assert_eq!(settlement_rail.attempts().len(), 2);
    assert_eq!(settlement_rail.completed_effect_count(), 1);
    require_completed(
        workflow
            .advance(
                started.instance().clone(),
                authority,
                &key("approved-payment:advance:completed"),
            )
            .expect("the applied workflow reaches its terminal"),
        "completed",
    );

    let observed = fixture
        .world
        .runtime
        .request(&approver, &scope)
        .on_branch(started.instance().branch())
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

fn require_assessment(outcome: WorkflowProgressOutcome, expected_node: &str) {
    match outcome {
        WorkflowProgressOutcome::AwaitingAssessment(required) => {
            assert_eq!(required.node_path(), expected_node);
            assert_eq!(required.query(), "payment_detail");
        }
        other => panic!("expected assessment at {expected_node}, got {other:?}"),
    }
}

fn require_completed(outcome: WorkflowProgressOutcome, expected_node: &str) {
    match outcome {
        WorkflowProgressOutcome::Completed(performed) => {
            assert_eq!(performed.node_path(), expected_node);
        }
        other => panic!("expected completed {expected_node}, got {other:?}"),
    }
}

fn key(value: &str) -> BankIdempotencyKey {
    BankIdempotencyKey::new(value).unwrap()
}
