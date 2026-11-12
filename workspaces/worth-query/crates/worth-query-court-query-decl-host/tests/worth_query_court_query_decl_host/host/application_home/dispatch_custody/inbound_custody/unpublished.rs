//! Continue the actual unpublished completion's original World identity after graceful close.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryInboundPendingReason;

#[test]
fn an_unpublished_authenticated_completion_continues_its_original_after_required_home_reopen() {
    authenticated_completion(false);
}

#[test]
fn an_unpublished_authenticated_completion_fences_new_effects_until_original_recovery() {
    authenticated_completion(true);
}

fn authenticated_completion(check_new_effect_fence: bool) {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(execution, true, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "unpublished-callback-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8441, &scope);
    let body = route.message(&owners, &effect, 71);
    let (cancel, stopped) = pending_support::scope();
    let pause = home
        .runtime()
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::new(1).unwrap());
    let receipt = std::thread::scope(|threads| {
        let controller = threads.spawn(move || {
            assert!(
                pause.wait_until_reached(Duration::from_secs(20)),
                "actual completion Native effect reaches World compare"
            );
            cancel.cancel();
            pause.release();
        });
        let receipt = home
            .runtime()
            .runtime()
            .receive_inbound_occurrence(&route.handle, &body, &stopped)
            .unwrap();
        controller.join().unwrap();
        receipt
    });
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(
        receipt.pending_reason(),
        Some(WorthQueryInboundPendingReason::RetainedUnpublished)
    );
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    assert_eq!(rail.contacts().len(), 1);
    drop(receipt);
    end_live_home_for_required_reopen(home, &execution);

    let (home, rail) = owners.open(execution, false, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "unpublished-callback-audience");
    if check_new_effect_fence {
        assert_new_effect_fenced(&home, 0x8443, &scope);
        assert!(
            rail.contacts().is_empty(),
            "a refused mutation cannot contact the rail"
        );
    }
    assert_eq!(route.message(&owners, &effect, 71), body);
    complete_original(&home, &route, effect.token);
    let original = terminal(&home, &route, &effect);
    if check_new_effect_fence {
        let _other_effect = emit(&home, 0x8443, &scope);
    }
    assert_eq!(receive(&home, &route, &body), Ok(Posture::AlreadyCompleted));
    assert_eq!(
        rail.contacts().len(),
        usize::from(check_new_effect_fence),
        "completion does not repeat physical contact"
    );
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "unpublished-callback-audience");
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}

#[test]
fn an_unpublished_transport_completion_continues_without_fabricating_a_live_dispatch_after_reopen()
{
    transport_completion(false);
}

#[test]
fn an_unpublished_transport_completion_fences_new_effects_until_original_recovery() {
    transport_completion(true);
}

fn transport_completion(check_new_effect_fence: bool) {
    let owners = Owners::new();
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let (home, rail) = owners.open(
        execution,
        true,
        &[Outcome::Acknowledged, Outcome::Completed],
    );
    let route = Route::install(&home, "unpublished-transport-audience");
    let (_, scope) = pending_support::scope();
    let effect = emit(&home, 0x8442, &scope);
    let correlation = effect.correlation;
    let (cancel, stopped) = pending_support::scope();
    let pause = home
        .runtime()
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::new(1).unwrap());
    std::thread::scope(|threads| {
        let controller = threads.spawn(move || {
            assert!(
                pause.wait_until_reached(Duration::from_secs(20)),
                "actual transport completion Native effect reaches World compare"
            );
            cancel.cancel();
            pause.release();
        });
        home.runtime()
            .runtime()
            .with_application_advancement(&stopped, |phase| {
                let owner = home
                    .runtime()
                    .runtime()
                    .observe_committed_dispatch_outbox_by_correlation(&phase, &correlation)
                    .unwrap();
                home.runtime()
                    .runtime()
                    .perform_committed_external_dispatch_for_test(
                        &phase.borrowed_request_for_test(),
                        &phase,
                        rail.as_ref(),
                        owner,
                        &stopped,
                    )
            })
            .unwrap()
            .unwrap();
        controller.join().unwrap();
    });
    assert!(home
        .runtime()
        .runtime()
        .observe_inbound_terminal(&route.handle, effect.token)
        .is_none());
    assert_eq!(
        custody(&home, &effect.token).observation(),
        Some(Observation::Completed)
    );
    assert_eq!(rail.contacts().len(), 2);
    end_live_home_for_required_reopen(home, &execution);

    let (home, rail) = owners.open(execution, false, &[Outcome::Acknowledged]);
    let route = Route::install(&home, "unpublished-transport-audience");
    if check_new_effect_fence {
        assert_new_effect_fenced(&home, 0x8444, &scope);
        assert!(
            rail.contacts().is_empty(),
            "a refused mutation cannot contact the rail"
        );
    }
    complete_original(&home, &route, effect.token);
    let original = terminal(&home, &route, &effect);
    if check_new_effect_fence {
        let _other_effect = emit(&home, 0x8444, &scope);
    }
    assert_eq!(
        receive(&home, &route, &route.message(&owners, &effect, 72)),
        Ok(Posture::AlreadyCompleted)
    );
    assert_eq!(
        rail.contacts().len(),
        usize::from(check_new_effect_fence),
        "completion does not repeat physical contact"
    );
    close(home, &execution);
    let (home, rail) = owners.open(execution, false, &[]);
    let route = Route::install(&home, "unpublished-transport-audience");
    assert_eq!(terminal(&home, &route, &effect), original);
    assert!(rail.contacts().is_empty());
    close(home, &execution);
}

// Each maintenance pass advances retained journal work once. Keep the exact
// original between physical Pending steps; never retry the transport.
fn complete_original(home: &Home, route: &Route, token: [u8; 32]) {
    for _ in 0..64 {
        let report = maintain(home, route);
        assert!(report.performed() <= 1);
        if report.performed() == 1 {
            assert_eq!(report.remaining(), 0);
            return;
        }
        assert_eq!(
            report.remaining(),
            1,
            "the original remains owned until completion"
        );
    }
    panic!(
        "the finite original disposition and occurrence did not complete: {:?}",
        home.runtime()
            .runtime()
            .observe_inbound_recovery_denial(&route.handle, token)
    );
}

fn assert_new_effect_fenced(home: &Home, key: u64, scope: &WorthQueryRequestScope) {
    use dispatch::inbound_action::InboundNoticeIntent;
    use worth_query_host::facade::primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationUncommitted,
    };
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        scope,
    );
    let outcome = runtime
        .request(&principal, scope)
        .advanced()
        .mutate(InboundNoticeIntent(key))
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("pending Native original must fence new effects: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
}
