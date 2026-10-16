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
        let ready = ReadyPaymentWorld::from_fixture(
            fixture::ordinary_read_world_with_two_approvers(
                "workflow-committed-custody",
                authentication::approval_configuration(),
            ),
            FaultScript::CommitThenLoseResponse,
        );
        let route = ready.install_synchronous_completion();
        let runtime = &ready.fixture.world.runtime;
        let vocabulary = runtime.approved_payment_workflow_runtime();
        let perform_key = key("approved-payment:operation:perform");
        let request = runtime
            .request(&ready.principal, &ready.scope)
            .on_branch(ready.operation.branch());
        let operation = runtime
            .approved_business_payment(&ready.principal, &ready.scope)
            .retain_apply(
                &ready.operation,
                ready.authority.clone(),
                perform_key.clone(),
            )
            .expect("retain the installed payment action and original occurrence");
        let mut operation = match operation.advance(&request) {
            worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Progress(operation) => operation,
            worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance::Refused { operation, denial } => {
                let _original_owner = operation;
                panic!("the original dispatch was refused: {denial:?}");
            }
            _ => panic!("the actual dispatch must retain its original owner until inbound"),
        };

        let token = ready.rail.attempts()[0]
            .token()
            .try_into()
            .expect("the real dispatch token has 32 bytes");
        assert!(
            runtime
                .observe_payment_rail_completion(&route, token)
                .is_none(),
            "a lost response cannot manufacture native inbound completion"
        );
        assert_eq!(ready.rail.completed_effect_count(), 1);
        // Complete through the existing original outbox handle and the real
        // rail's same-key reply, never by manufacturing an inbound message.
        ready
            .rail
            .under(FaultScript::Succeed, Duration::from_millis(150));
        let effect = loop {
            use worth_query_host::facade::application_entry::WorthQueryWorkflowOperationAdvance as Advance;
            match operation.advance(&request) {
                Advance::Progress(next) => operation = next,
                Advance::Completed(completed) => break completed,
                Advance::Refused { operation, denial } => {
                    let _original_owner = operation;
                    panic!("the completed real rail effect must settle: {denial:?}");
                }
                Advance::Terminal(terminal) => {
                    let _original_owner = terminal;
                    panic!("the completed real rail effect cannot stop terminally");
                }
            }
        };
        let dispatches = ready.rail.production_dispatches();
        assert_eq!(dispatches.len(), 2);
        assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
        assert_eq!(dispatches[0].payload, dispatches[1].payload);
        runtime
            .observe_payment_rail_completion(&route, token)
            .expect("the real same-key completion entered the installed terminal owner");
        // The actual original receipt is the actor oracle, never business input.
        let worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh {
            receipt: original_receipt,
            ..
        } = effect.evidence()
        else {
            panic!("the real rail payment must newly commit");
        };
        let original = original_receipt.principal_scope().principal();
        let executor = (
            original.partition_id(),
            original.local_slot(),
            original.generation(),
        );
        let later_actor = ready.fixture.authenticate(fixture::STRANGER);
        let command_key = key("approved-payment:operation:committed-custody");
        let prepared = runtime
            .request(&later_actor, &ready.scope)
            .mutate(ApprovedBusinessPaymentAdvanceIntent {
                // The advancer is authenticated independently; the original
                // payment payload still names the approver who performed it.
                input: ready.authority.clone(),
            })
            .without_source()
            .idempotency(&command_key)
            .prepare_workflow_advance(vocabulary, ready.instance.clone())
            .unwrap();
        reports();
        let before = reads();
        let accepted = prepared
            .accept_owned_operation(effect)
            .unwrap_or_else(|(_owner, cause)| {
                panic!("committed custody completes acceptance: {cause:?}")
            });
        let completed = match &accepted.progress {
            worth_query_host::facade::application_entry::WorkflowProgressOutcome::Completed(
                completed,
            ) => completed,
            other => panic!("committed custody must record its transition: {other:?}"),
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
        accepted
            .operation
            .close(&request)
            .unwrap_or_else(|(_owner, cause)| {
                panic!("original payment cleanup must close: {cause:?}")
            });
    }
}
