use std::sync::Arc;
use std::time::Duration;

use bank_domain::schema::ApprovePayment;
use bank_external_rail::{test_control::FaultScript, LedgerStatus, RailProcessHandle};
use bank_server::{BankApprovedPaymentApplyOutcome, BankAuthenticatedPrincipal};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::application_entry::{
    PublishedWorkflowInstanceRef, RequiredWorkflowOperation, WorkflowProgressOutcome,
    WorthQueryWorkflowOperationAcceptanceDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryExternalDispatchPostureKind,
};

use super::assertions::key;
use super::authentication::approval_configuration;
use super::fixture::{
    ordinary_read_world_with_approval_authentication, principal_id, OrdinaryReadFixture, APPROVER,
};
use super::journey::prepare_approved_payment_operation;
use super::rail_transport::{spawn_rail, BankEstateRailTransport};
use super::support::request_scope;

struct ReadyPaymentWorld {
    fixture: OrdinaryReadFixture,
    rail: Arc<BankEstateRailTransport>,
    _rail_process: RailProcessHandle,
    principal: BankAuthenticatedPrincipal,
    scope: WorthQueryRequestScope,
    authority: ApprovePayment,
    instance: PublishedWorkflowInstanceRef,
    operation: RequiredWorkflowOperation,
}

impl ReadyPaymentWorld {
    fn new(scenario: &str, script: FaultScript) -> Self {
        let fixture =
            ordinary_read_world_with_approval_authentication(scenario, approval_configuration());
        let rail_process = spawn_rail();
        let rail = Arc::new(BankEstateRailTransport::connected_to(
            rail_process.local_addr(),
            rail_process.test_control_addr(),
        ));
        rail.under(script, Duration::from_millis(150));
        fixture
            .world
            .runtime
            .install_external_effect_transport(rail.clone())
            .expect("the real rail transport installs");
        let principal = fixture.authenticate(APPROVER);
        let scope = request_scope();
        let authority = ApprovePayment {
            payment: fixture.payment,
            approver: principal_id(APPROVER),
        };
        let workflow = fixture
            .world
            .runtime
            .approved_business_payment(&principal, &scope);
        let (instance, operation) = prepare_approved_payment_operation(&workflow, &authority);
        Self {
            fixture,
            rail,
            _rail_process: rail_process,
            principal,
            scope,
            authority,
            instance,
            operation,
        }
    }

    fn perform(&self) -> BankApprovedPaymentApplyOutcome {
        self.fixture
            .world
            .runtime
            .approved_business_payment(&self.principal, &self.scope)
            .perform_apply(
                self.instance.clone(),
                &self.operation,
                self.authority.clone(),
                &key("approved-payment:operation:perform"),
            )
            .expect("the prepared Bank operation reaches Query's owner")
    }
}

#[test]
fn pending_rail_dispatch_remains_under_the_committed_outbox_owner() {
    let ready = ReadyPaymentWorld::new(
        "pending-rail-dispatch",
        FaultScript::AcknowledgeWithoutCompleting,
    );
    let performed = ready
        .perform()
        .into_performed()
        .expect("payment commits before rail completion");
    assert!(performed.co_committed_dispatch_outbox());
    assert_eq!(
        performed.external_dispatch_posture(),
        Some(WorthQueryExternalDispatchPostureKind::Acknowledged)
    );
    let attempts = ready.rail.attempts();
    assert_eq!(attempts.len(), 1);
    assert_eq!(
        ready.rail.ledger_status(&attempts[0]),
        LedgerStatus::Acknowledged
    );
    assert_eq!(ready.rail.completed_effect_count(), 0);
    let replay = ready
        .perform()
        .into_performed()
        .expect("the operation replay reads retained custody");
    assert!(!replay.newly_committed());
    assert_eq!(ready.rail.attempts().len(), 1);
    let workflow = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope);
    let denied = workflow
        .accept_applied(
            ready.instance.clone(),
            &ready.operation,
            ready.authority.clone(),
            &performed,
            &key("approved-payment:operation:accept:pending"),
        )
        .expect_err("pending rail settlement cannot complete the workflow operation");
    assert!(matches!(
        denied.denial::<WorthQueryWorkflowOperationAcceptanceDenial>(),
        Some(WorthQueryWorkflowOperationAcceptanceDenial::RecoveryRequired)
    ));
}

#[test]
fn lost_dispatch_before_rail_admission_requires_recovery_before_workflow_completion() {
    let ready = ReadyPaymentWorld::new(
        "lost-dispatch-before-admission",
        FaultScript::DisappearMidDispatch,
    );
    let performed = ready
        .perform()
        .into_performed()
        .expect("the Bank payment commits before the rail response is lost");
    assert!(performed.co_committed_dispatch_outbox());
    assert_eq!(
        performed.external_dispatch_posture(),
        Some(WorthQueryExternalDispatchPostureKind::Unresolved)
    );
    assert_eq!(ready.rail.attempts().len(), 1);
    assert_eq!(ready.rail.admission_count(), 0);
    assert_eq!(ready.rail.completed_effect_count(), 0);

    let workflow = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope);
    let denied = workflow
        .accept_applied(
            ready.instance.clone(),
            &ready.operation,
            ready.authority.clone(),
            &performed,
            &key("approved-payment:operation:accept:lost-dispatch"),
        )
        .expect_err("an unresolved dispatch cannot complete the workflow operation");
    assert!(matches!(
        denied.denial::<WorthQueryWorkflowOperationAcceptanceDenial>(),
        Some(WorthQueryWorkflowOperationAcceptanceDenial::RecoveryRequired)
    ));
    let required = match workflow
        .advance(
            ready.instance.clone(),
            ready.authority.clone(),
            &key("approved-payment:advance:lost-dispatch"),
        )
        .expect("the workflow retains the operation wait")
    {
        WorkflowProgressOutcome::AwaitingOperation(required) => required,
        other => panic!("expected the retained operation wait, got {other:?}"),
    };
    assert_eq!(
        required.transition_identity(),
        ready.operation.transition_identity()
    );

    ready
        .rail
        .under(FaultScript::Succeed, Duration::from_millis(150));
    let recovery = workflow
        .prepare_apply_recovery(
            &required,
            ready.authority.clone(),
            &performed,
            &key("approved-payment:operation:perform"),
        )
        .expect("the exact operation opens recovery")
        .safe_retry()
        .expect("recovery re-dispatches the retained outbox request");
    let accepted = workflow
        .accept_recovered_applied(
            ready.instance.clone(),
            &required,
            ready.authority.clone(),
            &performed,
            &recovery,
            &key("approved-payment:operation:accept:lost-dispatch:recovered"),
        )
        .expect("completed recovery settles the waiting operation");
    assert!(matches!(accepted, WorkflowProgressOutcome::Completed(_)));
    assert_eq!(ready.rail.attempts().len(), 2);
    assert_eq!(ready.rail.admission_count(), 1);
    assert_eq!(ready.rail.completed_effect_count(), 1);
    let dispatches = ready.rail.production_dispatches();
    assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
    assert_eq!(dispatches[0].payload, dispatches[1].payload);
    assert_eq!(
        ready.rail.ledger_status(&dispatches[0].correlation),
        LedgerStatus::Completed
    );
}

#[test]
fn unpublished_product_retains_owner_recovery_and_never_dispatches_the_rail() {
    let ready = ReadyPaymentWorld::new("unpublished-payment-product", FaultScript::Succeed);
    ready
        .fixture
        .world
        .runtime
        .application_program()
        .runtime()
        .fail_next_durable_append_for_test();
    let partial = match ready.perform() {
        BankApprovedPaymentApplyOutcome::Commit(
            WorthQueryApplicationCommitOutcome::ProductUnpublished(partial),
        ) => partial,
        other => {
            panic!("owner durability failure must retain unpublished product custody: {other:?}")
        }
    };
    assert!(partial.relational_requires_settlement());
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(ready.rail.attempts().is_empty());
    let recovery = partial.into_recovery();
    recovery
        .continue_owner_settlement()
        .expect("the original owner settles its performed facts");
    assert!(!recovery
        .inspect()
        .expect("recovery remains inspectable")
        .relational_requires_settlement());
    assert!(ready.rail.attempts().is_empty());
}
