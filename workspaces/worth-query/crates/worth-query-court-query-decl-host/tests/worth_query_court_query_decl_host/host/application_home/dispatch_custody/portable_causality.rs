//! Real recorded contacts keep one emission across fresh Home admissions.
use super::*;
use worth_query_host::facade::primary_graph::{
    ExternalEffectCorrelationIdentity, WorthQueryExternalEffectDispatch,
};

fn original(
    home: &Home,
) -> (
    ExternalEffectCorrelationIdentity,
    WorthQueryExternalEffectDispatch,
) {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(NotifyAccountBalance { balance: 31 })
        .without_source()
        .idempotency(&0x8c01)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("genuine original commit: {outcome:?}")
    };
    assert_eq!(result, 31);
    (
        *receipt.dispatch_outbox().unwrap().correlation(),
        receipt
            .external_dispatch()
            .expect("first physical send")
            .clone(),
    )
}

fn repeat(
    home: &Home,
    correlation: &ExternalEffectCorrelationIdentity,
    rail: &dyn WorthQueryExternalEffectTransport,
) -> WorthQueryExternalEffectDispatch {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let observed = home
                .runtime()
                .runtime()
                .observe_committed_dispatch_outbox_by_correlation(&phase, correlation)
                .unwrap();
            assert!(matches!(home.runtime().runtime().admit_external_dispatch_attempt_for_test(observed.clone()), Err(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_runtime::external_dispatch_attempt::WorthQueryExternalDispatchAdmissionDenial::DurableAttemptRecordRequired)), "a Home never enters the memory-only dispatch door");
            home.runtime()
                .runtime()
                .perform_committed_external_dispatch_for_test(
                    &phase.borrowed_request_for_test(), &phase, rail, observed, &scope,
                )
        })
        .unwrap()
        .expect("recorded, owner-admitted fresh contact")
}

#[test]
fn home_reopen_preserves_emission_causality_and_never_reuses_a_recorded_attempt() {
    let location = dispatch::location();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let rail = ScriptedRail::answering(&[Outcome::LostResponse, Outcome::LostResponse]);
    let home = open(&location, execution, &handlers, true, &rail);
    let (correlation, first) = original(&home);
    let first_record = custody(&home, correlation.bytes()).attempt();
    let second = repeat(&home, &correlation, rail.as_ref());
    let second_record = custody(&home, correlation.bytes()).attempt();
    assert_ne!(first_record, second_record);
    assert_eq!(
        first.causal_ladder().provider_commit(),
        second.causal_ladder().provider_commit()
    );
    assert_eq!(
        first.causal_ladder().emission(),
        second.causal_ladder().emission()
    );
    assert_ne!(
        first.causal_ladder().attempt(),
        second.causal_ladder().attempt()
    );
    assert_recorded_before_contact(&home, &rail.contacts()[1..]);
    Arc::into_inner(home)
        .expect("exclusive original Home")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("first close: {cause:?}"))
        .reclaim();
    let rail = ScriptedRail::answering(&[Outcome::LostResponse, Outcome::Completed]);
    let home = open(&location, execution, &handlers, false, &rail);
    let third = repeat(&home, &correlation, rail.as_ref());
    let third_record = custody(&home, correlation.bytes()).attempt();
    assert_eq!(third_record.home(), first_record.home());
    assert!(third_record.attempt_position() > second_record.attempt_position());
    assert_eq!(
        first.causal_ladder().provider_commit(),
        third.causal_ladder().provider_commit()
    );
    assert_eq!(
        first.causal_ladder().emission(),
        third.causal_ladder().emission()
    );
    assert_ne!(
        first.causal_ladder().attempt(),
        third.causal_ladder().attempt(),
        "fresh runtime must not restart the attempt name"
    );
    assert_ne!(
        second.causal_ladder().attempt(),
        third.causal_ladder().attempt()
    );
    let fourth = repeat(&home, &correlation, rail.as_ref());
    assert_eq!(
        fourth.posture().kind(),
        WorthQueryExternalDispatchPostureKind::Completed
    );
    assert_eq!(
        first.causal_ladder().provider_commit(),
        fourth.causal_ladder().provider_commit()
    );
    assert_eq!(
        first.causal_ladder().emission(),
        fourth.causal_ladder().emission()
    );
    assert_ne!(
        third.causal_ladder().attempt(),
        fourth.causal_ladder().attempt()
    );
    let fourth_record = custody(&home, correlation.bytes()).attempt();
    assert!(fourth_record.attempt_position() > third_record.attempt_position());
    assert_eq!(handlers.load(Ordering::SeqCst), 1);
    assert_eq!(rail.contacts().len(), 2);
    assert_recorded_before_contact(&home, &rail.contacts()[1..]);
    assert!(
        matches_current_owner(&home, &correlation, &fourth),
        "a genuine completion matches its own live runtime"
    );
    Arc::into_inner(home)
        .expect("exclusive reopened Home")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("second close: {cause:?}"))
        .reclaim();
    let rail = ScriptedRail::answering(&[]);
    let home = open(&location, execution, &handlers, false, &rail);
    assert_eq!(custody(&home, correlation.bytes()).attempt(), fourth_record);
    assert!(
        !matches_current_owner(&home, &correlation, &fourth),
        "portable meaning cannot turn an old live result into a fresh runtime observation"
    );
    assert!(rail.contacts().is_empty());
    assert_eq!(handlers.load(Ordering::SeqCst), 1);
    Arc::into_inner(home)
        .expect("exclusive third opening")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("third close: {cause:?}"))
        .reclaim();
}

fn matches_current_owner(
    home: &Home,
    correlation: &ExternalEffectCorrelationIdentity,
    dispatch: &WorthQueryExternalEffectDispatch,
) -> bool {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .with_application_advancement(&scope, |phase| {
            let owner = home
                .runtime()
                .runtime()
                .observe_committed_dispatch_outbox_by_correlation(&phase, correlation)
                .unwrap();
            dispatch.matches_committed_owner_for_test(
                home.runtime()
                    .runtime()
                    .runtime_for_test()
                    .authority_identity_for_test(),
                &owner,
            )
        })
        .unwrap()
}

fn source_commitment(
    home: &worth_query_host::facade::WorthQueryApplicationHome<
        fixture::schema::HomeIdentitySchema,
        fixture::HomeProgram,
    >,
) -> [u8; 32] {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let result = runtime
        .request(&principal, &scope)
        .query(fixture::balance_query::AccountRead(
            "account-001".to_owned(),
        ))
        .execute()
        .expect("actual selected Product read");
    assert_eq!(result.rows(), &[7]);
    result.observed_sources()[0]
        .output_source_epoch()
        .expect("selected Product source epoch")
        .runtime_idempotency_identity()
}

#[test]
fn home_source_commitment_survives_fresh_runtime_admission_and_separates_other_homes() {
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let request = fixture::request(
        location.clone(),
        build,
        Arc::new(AtomicUsize::new(0)),
        handlers.clone(),
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("original source Home: {cause:?}"));
    let original = source_commitment(&home);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("source close: {cause:?}"))
        .reclaim();
    let request =
        fixture::request_with_initializer(location, build, handlers.clone(), |_, _, _| {
            panic!("no reseeding on reopen")
        });
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("source reopen: {cause:?}"));
    assert_eq!(
        source_commitment(&home),
        original,
        "a fresh Query/World live instance cannot rename the same durable source"
    );
    assert_eq!(handlers.load(Ordering::SeqCst), 0);
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("source reclose: {cause:?}"))
        .reclaim();
    let request = fixture::request(
        fixture::location(),
        build,
        Arc::new(AtomicUsize::new(0)),
        handlers,
    );
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|cause| panic!("foreign source Home: {cause:?}"));
    assert_ne!(
        source_commitment(&home),
        original,
        "equivalent native values in another Home are distinct sources"
    );
    home.close(&execution)
        .unwrap_or_else(|cause| panic!("foreign source close: {cause:?}"))
        .reclaim();
}

struct UnwindingRail {
    home: Weak<Home>,
    observed: Mutex<Option<QueryDispatchCustody>>,
}
impl WorthQueryExternalEffectTransport for UnwindingRail {
    fn dispatch(&self, request: WorthQueryExternalDispatchRequest<'_>) -> Outcome {
        let home = self
            .home
            .upgrade()
            .expect("actual contact holds the live Home");
        *self.observed.lock().unwrap() = home
            .runtime()
            .runtime()
            .durable_dispatch_custody(request.correlation_token());
        panic!("the external owner loses its response by unwinding after contact")
    }
}

#[test]
fn unwinding_after_contact_preserves_unknown_custody_across_graceful_reopen() {
    let location = dispatch::location();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let rail = ScriptedRail::answering(&[Outcome::LostResponse]);
    let home = open(&location, execution, &handlers, true, &rail);
    let (correlation, _) = original(&home);
    let unwinding = UnwindingRail {
        home: Arc::downgrade(&home),
        observed: Mutex::new(None),
    };
    let lost = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        repeat(&home, &correlation, &unwinding)
    }));
    assert!(
        lost.is_err(),
        "the test must reach the actual external owner panic"
    );
    let recorded = unwinding
        .observed
        .lock()
        .unwrap()
        .expect("the real contact observed its recorded attempt");
    assert_eq!(recorded.observation(), None);
    assert_eq!(
        custody(&home, correlation.bytes()),
        recorded,
        "unwind retains its observation for request-funded settlement"
    );
    Arc::into_inner(home)
        .expect("unwind released live Home aliases")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("unwind close: {cause:?}"))
        .reclaim();
    let rail = ScriptedRail::answering(&[]);
    let home = open(&location, execution, &handlers, false, &rail);
    let recovered = custody(&home, correlation.bytes());
    assert_eq!(recovered.attempt(), recorded.attempt());
    assert_eq!(
        recovered.observation(),
        Some(Observation::Unresolved(
            QueryDispatchUnresolvedCause::UnknownProviderOutcome
        )),
        "contact unwinding cannot become proven no effect"
    );
    assert_eq!(handlers.load(Ordering::SeqCst), 1);
    assert!(rail.contacts().is_empty());
    Arc::into_inner(home)
        .expect("exclusive reopened Home")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("unwind reclose: {cause:?}"))
        .reclaim();
}
