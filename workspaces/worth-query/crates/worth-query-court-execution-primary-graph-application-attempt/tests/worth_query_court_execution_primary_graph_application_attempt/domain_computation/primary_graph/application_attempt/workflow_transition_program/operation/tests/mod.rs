use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::receipt::operation_receipt_requires_recovery;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::receipt::receipt_identity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::receipt::receipt_identity_from_outcome;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::workflow_transition_program::operation::receipt::validate_operation_receipt_custody;
use worth_query_execution::facade::domain_computation::safe_retry_recovery_handle;
use worth_query_execution::facade::domain_computation::WorthQueryExternalDispatchRequest;
use worth_query_execution::facade::domain_computation::WorthQueryExternalEffectTransport;
use worth_query_execution::facade::domain_computation::WorthQueryExternalTransportOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryRecoverySafeRetryAdmission;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::two_recoverable_application_commits;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;

pub struct CompletingTransport(AtomicUsize);

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
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "second-recoverable-commit",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .expect("the recoverable operation is installed");
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request_for_test())
                .authorize_operation(
                    &phase,
                    &principal,
                    &account,
                    &operation,
                    TypedMutationPreconditions::new(),
                    &request,
                )
                .expect("current truth admits recovery for both commits")
        })
        .expect("the fixture read owns its installed host call");

    let first = world
        .application
        .with_host_advancement(|phase| recover(&phase, &world, &first_receipt, &admission))
        .expect("the fixture read owns its installed host call");
    let second = world
        .application
        .with_host_advancement(|phase| recover(&phase, &world, &second_receipt, &admission))
        .expect("the fixture read owns its installed host call");
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
    assert_eq!(
        receipt_identity_from_outcome(
            first_receipt.runtime_authority_for_test().as_u64(),
            first_receipt
                .outcome_identity()
                .expect("committed outcome identity")
                .get(),
            first_receipt.installed_operation(),
        ),
        receipt_identity(&first_receipt).expect("the receipt derives the same identity")
    );
}

pub fn recover(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld,
    receipt: &worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt,
    admission: &worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation<
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput,
        worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account,
    >,
) -> WorthQueryRecoverySafeRetryAdmission {
    let handle = world
        .application
        .mint_recovery_handle(phase_custody, receipt)
        .expect("the committed receipt mints one recovery handle");
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, admission)
        .expect("current truth admits exact recovery authority");
    let redispatch = world
        .application
        .redispatch_admitted_external_effect(phase_custody, &handle, &authority, admission)
        .expect("the handle's committed outbox is re-dispatched");
    safe_retry_recovery_handle(phase_custody, handle, &authority, redispatch)
        .expect("completed re-dispatch mints the safe-retry proof")
}
