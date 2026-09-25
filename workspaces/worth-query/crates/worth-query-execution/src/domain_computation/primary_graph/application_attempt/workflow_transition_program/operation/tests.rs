use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use super::operation_receipt_requires_recovery;
use super::receipt::{receipt_identity, validate_operation_receipt_custody};
use crate::domain_computation::application_aftermath::{
    safe_retry_recovery_handle, WorthQueryExternalDispatchRequest,
    WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome,
    WorthQueryRecoverySafeRetryAdmission,
};
use crate::domain_computation::primary_graph::tests::{
    application_attempt::{authenticated_principal, resolved_account},
    fixture::{live_scope, ExactStatusRetentionOperation},
};
use crate::domain_computation::primary_graph::two_recoverable_application_commits;
use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenialKind;

struct CompletingTransport(AtomicUsize);

impl WorthQueryExternalEffectTransport for CompletingTransport {
    fn dispatch(
        &self,
        _request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.0.fetch_add(1, Ordering::AcqRel);
        WorthQueryExternalTransportOutcome::Completed
    }
}

#[test]
fn workflow_operation_custody_requires_the_exact_same_runtime_commit_proof() {
    let (world, first_receipt, second_receipt) = two_recoverable_application_commits(221, 222);
    let transport = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .expect("the recovery transport installs once");
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "second-recoverable-commit", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .expect("the recoverable operation is installed");
    let admission = world
        .selected_product()
        .authorize_operation(
            &principal,
            &account,
            &operation,
            TypedMutationPreconditions::new(),
            &request,
        )
        .expect("current truth admits recovery for both commits");

    let first = recover(&world, &first_receipt, &admission);
    let second = recover(&world, &second_receipt, &admission);
    assert_eq!(transport.0.load(Ordering::Acquire), 2);
    assert!(operation_receipt_requires_recovery(&first_receipt));

    let missing = validate_operation_receipt_custody("apply", &first_receipt, None)
        .expect_err("an unresolved receipt without recovery cannot settle");
    assert_eq!(
        missing.kind(),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    );
    let foreign = validate_operation_receipt_custody("apply", &first_receipt, Some(&second))
        .expect_err("another commit's genuine recovery proof cannot settle this receipt");
    assert_eq!(
        foreign.kind(),
        WorthQueryApplicationAttemptDenialKind::WorkflowTransitionAffinityMismatch
    );
    assert_eq!(
        validate_operation_receipt_custody("apply", &first_receipt, Some(&first))
            .expect("the exact recovery proof settles its receipt"),
        receipt_identity(&first_receipt).expect("the committed receipt has an outcome identity")
    );
}

fn recover(
    world: &crate::domain_computation::primary_graph::tests::fixture::AuthorizationWorld,
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation,
        crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionInput,
        crate::domain_computation::primary_graph::tests::fixture::Account,
    >,
) -> WorthQueryRecoverySafeRetryAdmission {
    let handle = world
        .application
        .mint_recovery_handle(receipt)
        .expect("the committed receipt mints one recovery handle");
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, admission)
        .expect("current truth admits exact recovery authority");
    let redispatch = world
        .application
        .redispatch_admitted_external_effect(&handle, &authority, admission)
        .expect("the handle's committed outbox is re-dispatched");
    safe_retry_recovery_handle(handle, &authority, redispatch)
        .expect("completed re-dispatch mints the safe-retry proof")
}
