//! A payment the rail owner holds stays in that owner's custody: no
//! cancellation, migration or program adoption disposes of it before the
//! owner settles.

use bank_external_rail::test_control::FaultScript;
use bank_server::{BankApplicationP1, BankApprovedPaymentWorkflowError};
use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceCancellationOutcome, WorkflowInstancePreparationDenial,
    WorthQueryWorkflowInstancePreparationDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationUncommitted, WorthQueryWorkflowInstanceCustody,
};

use super::assertions::key;
use super::ready_payment::ReadyPaymentWorld;

fn cancel(
    ready: &ReadyPaymentWorld,
    command_key: &str,
) -> Result<WorkflowInstanceCancellationOutcome, BankApprovedPaymentWorkflowError> {
    ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .prepare_cancellation(
            ready.instance.clone(),
            ready.authority.clone(),
            &key(command_key),
        )
        .map(|cancellation| cancellation.execute())
}

fn assert_in_owner_custody(
    outcome: Result<WorkflowInstanceCancellationOutcome, BankApprovedPaymentWorkflowError>,
) {
    match outcome {
        Err(BankApprovedPaymentWorkflowError::InstanceCancellation(
            WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
                WorkflowInstancePreparationDenial::Attempt(attempt),
            ),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
            "cancellation never disposes the custody the rail owner holds",
        ),
        other => panic!("a payment in owner custody must refuse cancellation: {other:?}"),
    }
}

/// A completed rail response alone cannot settle the workflow without the
/// authenticated inbound terminal.
fn owner_remains_pending(ready: &ReadyPaymentWorld, command_key: &str) {
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
            &key(command_key),
        )
        .expect_err("the rail response is not an authenticated inbound terminal");
    assert!(matches!(
        denied,
        BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(
            worth_query_host::facade::application_entry::WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                worth_query_host::facade::application_entry::WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            )
        )
    ));
}

#[test]
fn a_cancellation_stays_blocked_after_rail_completion_without_inbound() {
    let ready = ReadyPaymentWorld::new("cancel-in-owner-custody", FaultScript::Succeed);
    ready
        .perform()
        .into_performed()
        .expect("the payment commits into rail custody");
    assert_in_owner_custody(cancel(&ready, "approved-payment:cancel:in-custody"));
    owner_remains_pending(&ready, "approved-payment:operation:accept:before-cancel");
    assert_in_owner_custody(cancel(&ready, "approved-payment:cancel:without-inbound"));
    assert_eq!(ready.rail.completed_effect_count(), 1);
}

#[test]
fn a_cancellation_prepared_before_the_rail_takes_the_payment_goes_stale() {
    let ready = ReadyPaymentWorld::new("cancel-before-owner-custody", FaultScript::Succeed);
    let early = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope)
        .prepare_cancellation(
            ready.instance.clone(),
            ready.authority.clone(),
            &key("approved-payment:cancel:early"),
        )
        .expect("nothing is in the rail owner's custody yet");
    ready
        .perform()
        .into_performed()
        .expect("the payment commits into rail custody");
    match early.execute() {
        WorkflowInstanceCancellationOutcome::Application(
            WorthQueryApplicationUncommitted::Denied(denial),
        ) => assert_eq!(
            denial.kind(),
            WorthQueryApplicationCommitDenialKind::ProductBasisStale,
            "the custody the rail took moved the basis the cancellation read",
        ),
        other => panic!("a cancellation read before the rail took custody is stale: {other:?}"),
    }
    // The stale attempt claimed no key; the same key now meets the custody.
    assert_in_owner_custody(cancel(&ready, "approved-payment:cancel:early"));
}

#[test]
fn a_migration_stays_blocked_after_rail_completion_without_inbound() {
    let ready = ReadyPaymentWorld::new("migrate-in-owner-custody", FaultScript::Succeed);
    ready
        .perform()
        .into_performed()
        .expect("the payment commits into rail custody");
    let workflow = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope);
    let publish = |expected, command_key| match workflow
        .publish_definition(ready.authority.clone(), expected, &key(command_key))
        .expect("the payment definition publishes")
    {
        WorkflowDefinitionPublicationOutcome::Published(published) => {
            published.definition().clone()
        }
        other => panic!("expected definition publication, got {other:?}"),
    };
    // The fixture's own publication replays to name the running definition.
    let source = publish(
        WorkflowDefinitionExpectedPredecessor::Absent,
        "approved-payment:definition",
    );
    let successor = publish(
        WorkflowDefinitionExpectedPredecessor::Published(source),
        "approved-payment:definition:successor",
    );
    let migrate = |command_key| {
        workflow.migrate(
            ready.instance.clone(),
            successor.clone(),
            "completed",
            ready.authority.clone(),
            &key(command_key),
        )
    };
    match migrate("approved-payment:migrate:in-custody") {
        Err(BankApprovedPaymentWorkflowError::InstanceMigration(
            WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
                WorkflowInstancePreparationDenial::Attempt(attempt),
            ),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
            "migration never ends a source the rail owner still settles into",
        ),
        other => panic!("a payment in owner custody must refuse migration: {other:?}"),
    }
    owner_remains_pending(&ready, "approved-payment:operation:accept:before-migrate");
    match migrate("approved-payment:migrate:without-inbound") {
        Err(BankApprovedPaymentWorkflowError::InstanceMigration(
            WorthQueryWorkflowInstancePreparationDenial::InstancePreparation(
                WorkflowInstancePreparationDenial::Attempt(attempt),
            ),
        )) => assert_eq!(
            attempt.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowOperationInOwnerCustody,
            "rail completion without inbound acceptance still owns the operation",
        ),
        other => panic!("migration must name retained owner custody: {other:?}"),
    }
    assert_eq!(ready.rail.completed_effect_count(), 1);
}

#[test]
fn adoption_leaves_a_payment_the_rail_holds_with_no_disposition() {
    let ready = ReadyPaymentWorld::new("adopt-in-owner-custody", FaultScript::Succeed);
    ready
        .perform()
        .into_performed()
        .expect("the payment commits into rail custody");
    let runtime = &ready.fixture.world.runtime;
    let target = runtime
        .supported_program_revision::<BankApplicationP1>()
        .expect("Bank P1 is rostered beside P0");
    let programs = runtime
        .request(&ready.principal, &ready.scope)
        .on_branch(runtime.current_branch())
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let inventory = programs
        .adopt(&requirements)
        .workflow_inventory(4_096)
        .expect("the workflow inventory reads");
    let payment = inventory
        .instance(ready.instance.entity_id())
        .expect("the payment workflow is inventoried");
    assert_eq!(
        payment.custody(),
        &WorthQueryWorkflowInstanceCustody::OperationInOwnerCustody,
        "the rail owner settles the payment under the source program",
    );
    assert!(payment.legal_dispositions().is_empty());
}
