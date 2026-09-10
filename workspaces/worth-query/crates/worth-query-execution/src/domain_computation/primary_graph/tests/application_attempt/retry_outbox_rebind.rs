//! Final-source retry/outbox proof over the C7 application-attempt owner.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use super::super::fixture::AuthorizationWorld;
use super::preimage_evidence::{retained_status_program, RetentionMutationBreadth};
use super::{
    admitted_program, authenticated_principal, idempotency, installed_authorization_world,
    live_scope, resolved_account,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialStage, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationCommitTerminalKind,
};
use crate::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryExternalDispatchRequest,
    WorthQueryExternalEffectTransport, WorthQueryExternalTransportOutcome,
};

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
fn response_loss_retry_performs_no_second_managed_application_attempt() {
    let world = installed_authorization_world(true);
    let transport = Arc::new(CompletingTransport(AtomicUsize::new(0)));
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let original = commit_response_lost(&world, &transport);
    retry_without_managed_work(&world, &transport, &original);
    consume_armed_session_and_invariant_sentinels(&world);
    assert_unseen_commit_uses_the_managed_path(&world);
}

fn commit_response_lost(
    world: &AuthorizationWorld,
    transport: &CompletingTransport,
) -> WorthQueryApplicationCommitReceipt {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let account = resolved_account(world, "open", &request);
    let original = retained_status_program(
        world,
        &principal,
        &account,
        &request,
        "response-lost",
        RetentionMutationBreadth::Narrow,
    );
    world.faults.lose_next_commit_response();
    let before_commit = world.application.application_attempt_work();
    let WorthQueryApplicationCommitOutcome::Committed(original) = world
        .application
        .compare_and_commit_application(original, idempotency(201, 202))
    else {
        panic!("response-loss recovery must return the authoritative commit");
    };
    let committed_work = world
        .application
        .application_attempt_work()
        .since(before_commit);
    assert!(original.dispatch_outbox().is_some());
    assert_eq!(committed_work.external_dispatch_admissions, 1);
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert_eq!(
        original.terminal().kind(),
        WorthQueryApplicationCommitTerminalKind::Executed
    );
    assert_eq!(original.terminal().attempt_resources_released(), Some(true));
    original
}

fn retry_without_managed_work(
    world: &AuthorizationWorld,
    transport: &CompletingTransport,
    original: &WorthQueryApplicationCommitReceipt,
) {
    let retry_request = live_scope();
    let retry_principal = authenticated_principal(world, &retry_request);
    let retry_account = resolved_account(world, "response-lost", &retry_request);
    let retry = retained_status_program(
        world,
        &retry_principal,
        &retry_account,
        &retry_request,
        "response-lost",
        RetentionMutationBreadth::Narrow,
    );
    world.faults.reject_next_session_prepare();
    world.faults.skip_next_invariant_owner_execution();
    let before_retry = world.application.application_attempt_work();
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) = world
        .application
        .compare_and_commit_application(retry, idempotency(201, 202))
    else {
        panic!("equivalent retry must resolve the response-lost commit");
    };
    let retry_work = world
        .application
        .application_attempt_work()
        .since(before_retry);
    assert_eq!(retry_work.retained_resolutions, 1);
    assert_no_managed_attempt_work(retry_work);
    assert_eq!(transport.0.load(Ordering::Acquire), 1);
    assert!(recovered.is_same_authoritative_commit(original));
    assert_eq!(recovered.dispatch_outbox(), original.dispatch_outbox());
    assert_eq!(recovered.terminal().attempt_resources_released(), None);
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

fn consume_armed_session_and_invariant_sentinels(world: &AuthorizationWorld) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let probe_account = resolved_account(world, "unrelated", &request);
    let rejected_session = admitted_program(
        world,
        &principal,
        &probe_account,
        &request,
        "session-sentinel",
    );
    let WorthQueryApplicationCommitOutcome::Denied(session_denial) = world
        .application
        .compare_and_commit_application(rejected_session, idempotency(203, 204))
    else {
        panic!("the retry must leave the provider-session sentinel armed");
    };
    assert_eq!(
        session_denial.stage(),
        WorthQueryApplicationCommitDenialStage::ProviderPlan
    );
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let probe_account = resolved_account(world, "unrelated", &request);
    let rejected_invariant = admitted_program(
        world,
        &principal,
        &probe_account,
        &request,
        "invariant-sentinel",
    );
    let WorthQueryApplicationCommitOutcome::Denied(invariant_denial) = world
        .application
        .compare_and_commit_application(rejected_invariant, idempotency(205, 206))
    else {
        panic!("the retry must leave the invariant-owner sentinel armed");
    };
    assert_eq!(
        invariant_denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
}

fn assert_unseen_commit_uses_the_managed_path(world: &AuthorizationWorld) {
    let request = live_scope();
    let principal = authenticated_principal(world, &request);
    let probe_account = resolved_account(world, "unrelated", &request);
    let positive_control = admitted_program(
        world,
        &principal,
        &probe_account,
        &request,
        "positive-control",
    );
    let before_control = world.application.application_attempt_work();
    assert!(matches!(
        world
            .application
            .compare_and_commit_application(positive_control, idempotency(207, 208)),
        WorthQueryApplicationCommitOutcome::Committed(_)
    ));
    let control = world
        .application
        .application_attempt_work()
        .since(before_control);
    assert_eq!(
        control.retained_resolutions, 3,
        "an unseen commit checks idempotency before registration, after registration, and under commit authority"
    );
    assert_eq!(control.managed_bridge_plans, 1);
    assert_eq!(control.provider_session_readmissions, 1);
    assert_eq!(control.provider_session_preparations, 1);
    assert_eq!(control.attempt_registrations, 1);
    assert_eq!(control.overlay_stagings, 1);
    assert_eq!(control.invariant_state_loads, 1);
    assert_eq!(control.invariant_executions, 1);
    assert_eq!(control.staged_session_preparations, 1);
    assert_eq!(control.prepared_commits, 1);
    assert_eq!(control.attempt_aborts, 0);
    assert_eq!(control.managed_cleanups, 1);
    assert_eq!(control.external_dispatch_admissions, 0);
    assert_eq!(world.application.provider_session_resource_count(), 0);
}

fn assert_no_managed_attempt_work(
    work: crate::domain_computation::primary_graph::provider::WorthQueryApplicationAttemptWorkSnapshot,
) {
    assert_eq!(work.managed_bridge_plans, 0);
    assert_eq!(work.provider_session_readmissions, 0);
    assert_eq!(work.provider_session_preparations, 0);
    assert_eq!(work.staged_session_preparations, 0);
    assert_eq!(work.attempt_registrations, 0);
    assert_eq!(work.overlay_stagings, 0);
    assert_eq!(work.invariant_state_loads, 0);
    assert_eq!(work.invariant_executions, 0);
    assert_eq!(work.prepared_commits, 0);
    assert_eq!(work.attempt_aborts, 0);
    assert_eq!(work.managed_cleanups, 0);
    assert_eq!(work.external_dispatch_admissions, 0);
}
