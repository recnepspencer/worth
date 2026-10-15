//! Workflow owner doors refuse before their authority reader and open exactly once.
#[path = "../../tests/approved_payment_workflow/approval.rs"]
mod approval;
#[path = "../../tests/approved_payment_workflow/assertions.rs"]
mod assertions;
#[path = "../../tests/approved_payment_workflow/authentication.rs"]
mod authentication;
#[path = "../../tests/ordinary_reads/fixture.rs"]
mod fixture;
#[path = "../../tests/approved_payment_workflow/journey.rs"]
mod journey;
#[path = "../../tests/ordinary_mutations/estate_operations/external_effect_dispatch/rail_transport.rs"]
mod rail_transport;
#[path = "../../tests/approved_payment_workflow/ready_payment.rs"]
mod ready_payment;
use crate::support;
use assertions::key;
use bank_domain::schema::{
    ApprovedBusinessPaymentAdvanceIntent, ApprovedBusinessPaymentApplyIntent,
};
use bank_external_rail::test_control::FaultScript;
use ready_payment::ReadyPaymentWorld;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
    },
    application_entry::{
        WorthQueryApplicationRecoveryRequestDenial as Recovery,
        WorthQueryWorkflowOperationAcceptanceDenial as CustodyAcceptance,
        WorthQueryWorkflowOperationOwnerAcceptanceDenial as Acceptance,
        WorthQueryWorkflowOperationOwnerPosture as Posture,
        WorthQueryWorkflowOperationRecoveryPreparationDenial as Preparation,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationAttemptDenialKind as AttemptKind,
        WorthQueryExecutionPlacementForTest as Placement,
        WorthQueryManagedApplicationRecoveryDenial as NativeRecovery,
    },
};
struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}
fn zero_work() {
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        64 * 1_024 * 1_024,
        0,
    )));
}
fn opening_refused(denial: Recovery, before: u64) {
    assert!(matches!(
        denial,
        Recovery::Recovery(NativeRecovery::ExecutionDenied(Denial::Resource(
            Resource::WorkExhausted
        )))
    ));
    assert_eq!(reads(), before, "refused opening must precede every reader");
    assert_eq!(reports().len(), 1);
    bound(None);
}
fn admitted(before: u64) {
    assert!(
        reads() > before,
        "the admitted call reaches real authority reads"
    );
    let opened = reports();
    assert_eq!(opened.len(), 1, "one request, including custody acceptance");
    assert!(opened[0].is_ok(), "admitted request keeps its report");
}
#[test]
fn owner_acceptance_and_recovery_doors_open_before_their_first_reader() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let ready = ReadyPaymentWorld::new(
            "workflow-request-opening",
            FaultScript::CommitThenLoseResponse,
        );
        ready
            .perform()
            .into_performed()
            .expect("the real rail holds this committed payment");
        let runtime = &ready.fixture.world.runtime;
        let vocabulary = runtime.approved_payment_workflow_runtime();
        let perform_key = key("approved-payment:operation:perform");
        let command_key = key("approved-payment:operation:opening-proof");
        let operation = || {
            runtime
                .request(&ready.principal, &ready.scope)
                .on_branch(ready.operation.branch())
                .mutate(ApprovedBusinessPaymentApplyIntent {
                    input: ready.authority.clone(),
                })
                .idempotency(&perform_key)
                .for_workflow_operation_recovery(vocabulary, &ready.operation)
                .unwrap()
        };
        let advance = || {
            runtime
                .request(&ready.principal, &ready.scope)
                .mutate(ApprovedBusinessPaymentAdvanceIntent {
                    input: ready.authority.clone(),
                })
                .without_source()
                .idempotency(&command_key)
                .prepare_workflow_advance(vocabulary, ready.instance.clone())
                .unwrap()
        };
        let prepared = advance();
        let mutation = operation();
        reports();
        zero_work();
        let before = reads();
        let denied = prepared
            .accept_operation_from_owner(&ready.operation, mutation)
            .unwrap_err();
        let Acceptance::Opening(denial) = denied else {
            panic!("opening cause: {denied:?}")
        };
        opening_refused(denial, before);

        let prepared = advance();
        let mutation = operation();
        reports();
        let before = reads();
        assert!(matches!(
            prepared.accept_operation_from_owner(&ready.operation, mutation),
            Err(Acceptance::Owner(Posture::DispatchPending))
        ));
        admitted(before);

        let mutation = operation();
        reports();
        zero_work();
        let before = reads();
        let denied = mutation
            .advanced()
            .prepare_workflow_operation_recovery_from_owner(&ready.operation)
            .err()
            .unwrap();
        let Preparation::Opening(denial) = denied else {
            panic!("opening cause: {denied:?}")
        };
        opening_refused(denial, before);
        let mutation = operation();
        reports();
        let before = reads();
        let prepared = mutation
            .advanced()
            .prepare_workflow_operation_recovery_from_owner(&ready.operation)
            .expect("the genuine unresolved outbox admits recovery");
        admitted(before);
        ready
            .rail
            .under(FaultScript::Succeed, Duration::from_millis(150));
        let recovery = vocabulary
            .runtime()
            .with_host_advancement(|phase| prepared.safe_retry(&phase))
            .expect("fixture retry owns its host call")
            .expect("the retained real payment safely retries");

        let prepared = advance();
        let mutation = operation();
        reports();
        zero_work();
        let before = reads();
        let denied = prepared
            .accept_recovered_operation_from_owner(&ready.operation, mutation, &recovery)
            .unwrap_err();
        let Acceptance::Opening(denial) = denied else {
            panic!("opening cause: {denied:?}")
        };
        opening_refused(denial, before);
        let prepared = advance();
        let mutation = operation();
        reports();
        let before = reads();
        let outcome =
            prepared.accept_recovered_operation_from_owner(&ready.operation, mutation, &recovery);
        assert!(
            matches!(outcome, Err(Acceptance::Acceptance(CustodyAcceptance::Attempt(ref denial)))
            if denial.kind() == AttemptKind::WorkflowTransitionAffinityMismatch),
            "real recovered owner: {outcome:?}"
        );
        admitted(before);
    }
}

#[test]
fn committed_owner_custody_is_accepted_in_one_request() {
    let _observation = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let ready = ReadyPaymentWorld::with_two_approvers(
            "workflow-committed-custody",
            FaultScript::Succeed,
        );
        let route = ready.install_synchronous_completion();
        let runtime = &ready.fixture.world.runtime;
        let vocabulary = runtime.approved_payment_workflow_runtime();
        let perform_key = key("approved-payment:operation:perform");
        // Retain the actual native performed receipt, rather than reconstructing
        // executor identity from a business-input principal or a later command.
        let effect = runtime
            .request(&ready.principal, &ready.scope)
            .on_branch(ready.operation.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent {
                input: ready.authority.clone(),
            })
            .idempotency(&perform_key)
            .for_workflow_operation(vocabulary, &ready.operation)
            .unwrap()
            .execute_in_program(
                vocabulary.program_runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("the real native payment executes");
        let worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed {
            receipt: original_receipt, ..
        } = effect else { panic!("the real rail payment must newly commit: {effect:?}"); };
        let original = original_receipt.principal_scope().principal();
        let executor = (
            original.partition_id(),
            original.local_slot(),
            original.generation(),
        );

        let token = ready.rail.attempts()[0]
            .token()
            .try_into()
            .expect("the real dispatch token has 32 bytes");
        ready
            .fixture
            .world
            .runtime
            .observe_payment_rail_completion(&route, token)
            .expect("the synchronous transport completion entered the installed terminal owner");
        let later_actor = ready.fixture.authenticate(fixture::STRANGER);
        let later_authority = bank_domain::schema::ApprovePayment {
            payment: ready.fixture.payment,
            approver: fixture::principal_id(fixture::STRANGER),
        };
        let command_key = key("approved-payment:operation:committed-custody");
        let mutation = runtime
            .request(&ready.principal, &ready.scope)
            .on_branch(ready.operation.branch())
            .mutate(ApprovedBusinessPaymentApplyIntent {
                input: ready.authority.clone(),
            })
            .idempotency(&perform_key)
            .for_workflow_operation_recovery(vocabulary, &ready.operation)
            .unwrap();
        let prepared = runtime
            .request(&later_actor, &ready.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent {
                input: later_authority,
            })
            .without_source()
            .idempotency(&command_key)
            .prepare_workflow_advance(vocabulary, ready.instance.clone())
            .unwrap();
        reports();
        let before = reads();
        let outcome = prepared.accept_operation_from_owner(&ready.operation, mutation);
        let completed = match outcome {
            Ok(
                worth_query_host::facade::application_entry::WorkflowProgressOutcome::Completed(
                    completed,
                ),
            ) => completed,
            other => panic!("committed custody completes acceptance: {other:?}"),
        };
        let later = completed.receipt().principal_scope().principal();
        let advancer = (later.partition_id(), later.local_slot(), later.generation());
        assert_ne!(
            executor, advancer,
            "distinct real admitted actors are essential"
        );
        use worth_query_host::facade::declaration::application_capability::ApplicationWorkflowRecordedRole as Role;
        let changes = completed.receipt().committed_changes();
        assert_eq!(
            changes
                .workflow_actor_fields_for_test(completed.transition(), Role::Executor)
                .unwrap()
                .map(|actor| (
                    actor.partition_value(),
                    actor.local_slot_value(),
                    actor.generation_value()
                )),
            Some(executor)
        );
        assert_eq!(
            changes
                .workflow_actor_fields_for_test(completed.transition(), Role::Advancer)
                .unwrap()
                .map(|actor| (
                    actor.partition_value(),
                    actor.local_slot_value(),
                    actor.generation_value()
                )),
            Some(advancer)
        );
        assert_eq!(ready.rail.completed_effect_count(), 1);
        admitted(before);
    }
}
