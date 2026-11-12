//! Authentic port-free grants meet a new filesystem Home receiver before effects.
use super::*;
use worth_query_host::facade::{
    certification::WorthQueryCertificationCostRuntimeExt,
    primary_graph::{
        WorthQueryApplicationUncommitted, WorthQueryProviderSessionDenialKind as Kind,
    },
};

#[test]
fn reopened_home_refuses_the_authentic_prior_provider_session_before_any_provider_effect() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let path = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            fixture::request(path.clone(), build, initialized.clone(), contacts.clone())
                .open_in_advancement(&phase)
                .unwrap_or_else(|refused| opening_failed("session writer", &refused)),
        )
    })
    .unwrap();
    let (_, live) = scope();
    let capture = home
        .runtime()
        .runtime()
        .capture_next_provider_session_grant_for_test()
        .unwrap();
    assert!(matches!(
        set(home.runtime(), &live, 0x6772616e74, 8).unwrap(),
        Outcome::Commit(WorthQueryApplicationUncommitted::Denied(_))
    ));
    let grant = capture
        .take_captured()
        .expect("the actual owner issued this grant");
    assert_eq!(grant.counters().provider_calls(), 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    // Both the authentic grant AND the control survive graceful Home close.
    // Neither owns a source port; no capability is released by the fixture.
    (*home).close(&execution).unwrap_or_else(|cause| {
        panic!("session writer closes with its port-free grant retained: {cause:?}")
    });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(
            fixture::request_with_initializer(path, build, contacts.clone(), |_, _, _| {
                panic!("existing session Home cannot replay its initializer")
            })
            .open_in_advancement(&phase)
            .unwrap_or_else(|refused| opening_failed("session receiver", &refused)),
        )
    })
    .unwrap();
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    let (_, live) = scope();
    let app = home.runtime().runtime();
    let before = app
        .with_application_advancement(&live, |phase| {
            app.capture_certification_cost_scope(&phase, app.current_world())
                .unwrap()
        })
        .unwrap();
    let refusal = app
        .with_application_advancement(&live, |phase| {
            app.prepare_provider_session_grant(&phase, grant, || {
                panic!("stale session cannot construct a live run or evaluate a supplier")
            })
            .unwrap_err()
        })
        .unwrap();
    assert_eq!(refusal.kind(), Kind::StaleIncarnation);
    assert_eq!(refusal.counters().provider_calls(), 0);
    let work = app
        .with_application_advancement(&live, |phase| {
            app.observe_certification_cost(&phase, &before)
                .unwrap()
                .application_work()
        })
        .unwrap();
    assert_eq!(work.provider_session_readmissions(), 0);
    assert_eq!(work.provider_session_preparations(), 0);
    assert_eq!(work.attempt_registrations(), 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 7);
    assert!(matches!(
        set(home.runtime(), &live, 0x63757272656e74, 8).unwrap(),
        Outcome::Committed { result: 8, .. }
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    assert_eq!(balance(home.runtime(), &live), 8);
    drop(capture);
}

#[test]
fn foreign_home_session_grant_is_refused_before_provider_preparation() {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let contacts = Arc::new(AtomicUsize::new(0));
    let foreign = pending_support::open(execution, contacts.clone());
    let (_, live) = scope();
    let capture = foreign
        .runtime()
        .runtime()
        .capture_next_provider_session_grant_for_test()
        .unwrap();
    assert!(matches!(
        set(foreign.runtime(), &live, 123, 8).unwrap(),
        Outcome::Commit(WorthQueryApplicationUncommitted::Denied(_))
    ));
    let grant = capture.take_captured().unwrap();
    drop(foreign);
    let home = pending_support::open(execution, contacts.clone());
    let app = home.runtime().runtime();
    let before = app
        .with_application_advancement(&live, |phase| {
            app.capture_certification_cost_scope(&phase, app.current_world())
                .unwrap()
        })
        .unwrap();
    let refusal = app
        .with_application_advancement(&live, |phase| {
            app.prepare_provider_session_grant(&phase, grant, || {
                panic!("foreign session cannot construct a live run or evaluate a supplier")
            })
            .unwrap_err()
        })
        .unwrap();
    assert_eq!(refusal.kind(), Kind::ForeignGraphAuthority);
    assert_eq!(refusal.counters().provider_calls(), 0);
    let work = app
        .with_application_advancement(&live, |phase| {
            app.observe_certification_cost(&phase, &before)
                .unwrap()
                .application_work()
        })
        .unwrap();
    assert_eq!(work.provider_session_readmissions(), 0);
    assert_eq!(work.provider_session_preparations(), 0);
    assert_eq!(work.attempt_registrations(), 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 7);
    assert!(matches!(
        set(home.runtime(), &live, 789, 8).unwrap(),
        Outcome::Committed { result: 8, .. }
    ));
    assert_eq!(contacts.load(Ordering::SeqCst), 2);
    drop(capture);
}
