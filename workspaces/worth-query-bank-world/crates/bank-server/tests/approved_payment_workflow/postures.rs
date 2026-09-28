use std::num::NonZeroUsize;
use std::time::Duration;

use bank_external_rail::{test_control::FaultScript, LedgerStatus};
use bank_server::BankApprovedPaymentApplyOutcome;
use worth_query_host::facade::application_entry::{
    WorkflowProgressOutcome, WorthQueryWorkflowOperationOwnerAcceptanceDenial,
    WorthQueryWorkflowOperationOwnerPosture, WorthQueryWorkflowOperationRecoveryPreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryExternalDispatchPostureKind,
};

use super::assertions::key;
use super::ready_payment::ReadyPaymentWorld;

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
            ready.authority.clone(),
            &key("approved-payment:operation:perform"),
            &key("approved-payment:operation:accept:pending"),
        )
        .expect_err("pending rail settlement cannot complete the workflow operation");
    assert!(matches!(
        denied,
        bank_server::BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(
            WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            )
        )
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
            ready.authority.clone(),
            &key("approved-payment:operation:perform"),
            &key("approved-payment:operation:accept:lost-dispatch"),
        )
        .expect_err("an unresolved dispatch cannot complete the workflow operation");
    assert!(matches!(
        denied,
        bank_server::BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(
            WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            )
        )
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
            ready.authority.clone(),
            &key("approved-payment:operation:perform"),
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
        .fail_next_durable_append_for_test();
    let partial = match ready.perform() {
        BankApprovedPaymentApplyOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductUnpublished(partial),
        ) => partial,
        other => {
            panic!("owner durability failure must retain unpublished product custody: {other:?}")
        }
    };
    assert!(partial.relational_requires_settlement());
    assert_eq!(partial.owner_effect_count(), 1);
    assert!(ready.rail.attempts().is_empty());
    let workflow = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope);
    let denied = workflow
        .prepare_apply_recovery(
            &ready.operation,
            ready.authority.clone(),
            &key("approved-payment:operation:perform"),
        )
        .err()
        .expect("outbox recovery cannot precede owner product publication");
    assert!(matches!(
        denied,
        bank_server::BankApprovedPaymentWorkflowError::OperationRecoveryPreparation(
            WorthQueryWorkflowOperationRecoveryPreparationDenial::Owner(
                WorthQueryWorkflowOperationOwnerPosture::ProductUnpublished(_)
            )
        )
    ));
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

#[test]
fn indeterminate_product_comparison_retains_custody_without_rail_dispatch() {
    let ready = ReadyPaymentWorld::new("indeterminate-payment-product", FaultScript::Succeed);
    ready
        .fixture
        .world
        .runtime
        .panic_before_product_compare_once_for_test();
    let unresolved = match ready.perform() {
        BankApprovedPaymentApplyOutcome::Commit(
            WorthQueryApplicationUncommitted::Indeterminate(unresolved),
        ) => unresolved,
        other => panic!("World comparison unwind must retain indeterminate custody: {other:?}"),
    };
    assert!(!unresolved.detail().is_empty());
    assert!(ready.rail.attempts().is_empty());
    let owner = &ready.fixture.world.runtime;
    let page = owner
        .product_publication_recovery_page(None, NonZeroUsize::new(1).unwrap())
        .expect("World retains a bounded recovery catalog for unresolved owner work");
    let [row] = page.rows() else {
        panic!("the exact unresolved owner movement must be discoverable");
    };
    let recovery = owner
        .readmit_product_publication_recovery(row.handle())
        .expect("the same Query owner readmits its World recovery record");
    assert_eq!(
        recovery
            .inspect()
            .expect("retained owner evidence remains inspectable")
            .owner_effect_count(),
        1
    );
    assert!(ready.rail.attempts().is_empty());
}
