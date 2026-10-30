//! A payment the rail owner holds stays in that owner's custody: no
//! cancellation, migration or program adoption disposes of it before the
//! owner settles.

#![allow(
    clippy::result_large_err,
    reason = "Custody courts retain the original native workflow refusal and cleanup owner"
)]

pub use bank_external_rail::test_control::FaultScript;
pub use bank_server::{BankApplicationP1, BankApprovedPaymentWorkflowError};
pub use worth_query_host::facade::application_entry::{
    WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
    WorkflowInstanceCancellationOutcome, WorkflowInstancePreparationDenial,
    WorthQueryWorkflowInstancePreparationDenial,
};
pub use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationUncommitted,
};

pub use super::assertions::key;
pub use super::ready_payment::ReadyPaymentWorld;

#[path = "owner_custody/correction.rs"]
pub mod correction;

pub fn cancel(
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

pub fn assert_in_owner_custody(
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
pub fn owner_remains_pending(ready: &ReadyPaymentWorld, command_key: &str) {
    let workflow = ready
        .fixture
        .world
        .runtime
        .approved_business_payment(&ready.principal, &ready.scope);
    let denied = workflow
        .advanced()
        .accept_native_effect(
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
        BankApprovedPaymentWorkflowError::OperationOwnerAcceptance(denial) if matches!(denial.as_ref(), worth_query_host::facade::application_entry::WorthQueryWorkflowOperationOwnerAcceptanceDenial::Owner(
                worth_query_host::facade::application_entry::WorthQueryWorkflowOperationOwnerPosture::DispatchPending
            ))
    ));
}
