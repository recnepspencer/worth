//! Original request cancellation crosses every real durable receipt read stage.
use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementPhase, WorthQueryManagedComputationInterruption as Interruption,
    },
    application_entry::{
        WorthQueryApplicationRecoveryRequestDenial as RecoveryDenial,
        WorthQueryApplicationRequestMutationDenial as RequestDenial,
    },
    primary_graph::{
        WorthQueryApplicationIdempotencyResolutionDenialKind as Idempotency,
        WorthQueryManagedApplicationRecoveryDenial as Recovery,
        WorthQueryProviderSessionControlStopKind as Control,
    },
};

pub(super) fn assert_cancelled(answer: Result<Outcome<(), u64>, RequestDenial>) {
    match answer {
        Ok(Outcome::Cancelled) => {},
        Err(RequestDenial::Idempotency(denial)) => match denial.kind() {
            Idempotency::ExecutionDenied(Err(Control::Cancelled))
            | Idempotency::Authorization(worth_query_host::facade::primary_graph::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(Control::Cancelled))) => {},
            other => panic!("the exact original control stop survives its admission stage: {other:?}"),
        },
        Err(denial) => assert!(matches!(RecoveryDenial::from(denial),
            RecoveryDenial::Recovery(Recovery::ExecutionDenied(WorthQueryAdvancementDenial::Interrupted(Interruption::Cancelled))))),
        other => panic!("the original cancellation must remain typed: {other:?}"),
    }
}

#[test]
fn cancelled_directory_key_without_effect_executes_once_on_retry() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let home = pending_support::open(
        worth_execution::ExecutionRequest::leased(&lease),
        contacts.clone(),
    );
    let (cancel, stopped) = scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &stopped,
    );
    cancel.cancel();
    assert_cancelled(
        home.runtime()
            .request(&principal, &stopped)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&0xcace1_u64)
            .execute_in_program(
                home.runtime(),
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            ),
    );
    let (_, live) = scope();
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
    assert_eq!(balance(home.runtime(), &live), 7);
    let original = match set(home.runtime(), &live, 0xcace1, 8).unwrap() {
        Outcome::Committed { receipt, result: 8 } => receipt,
        other => panic!("the proven no-effect key executes once: {other:?}"),
    };
    let recorded = match set(home.runtime(), &live, 0xcace1, 8).unwrap() {
        Outcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("the retry reads the original durable receipt: {other:?}"),
    };
    receipt_fields::assert_live_agrees(&original, &recorded);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    drop((original, recorded));
    drop(home);
}

#[test]
fn original_directory_receipt_preserves_cancellation_at_every_consult() {
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let home = pending_support::open(
        worth_execution::ExecutionRequest::leased(&lease),
        contacts.clone(),
    );
    let (_, live) = scope();
    let fresh = set(home.runtime(), &live, 0xcace2, 8).unwrap();
    assert!(matches!(fresh, Outcome::Committed { result: 8, .. }));
    fn call(
        runtime: &worth_query_host::facade::application_installation::WorthQueryProgramApplicationRuntime<fixture::schema::HomeIdentitySchema, fixture::HomeProgram>,
        phase: &WorthQueryAdvancementPhase<'_>,
        scope: &WorthQueryRequestScope,
        principal: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<fixture::schema::HomeIdentitySchema>,
    ) -> Result<Outcome<(), u64>, RequestDenial> {
        runtime
            .request(principal, scope)
            .advanced()
            .mutate(fixture::native_mutation::SetAccountBalance {
                account: "account-001".to_owned(),
                balance: 8,
            })
            .without_source()
            .idempotency(&0xcace2_u64)
            .execute_in_program_in_advancement(phase, runtime)
    }
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &live,
    );
    // Count only the admitted business callback. Directory upkeep and the
    // completion poll preserve an already-produced result by contract.
    let (healthy, consults) = home
        .runtime()
        .runtime()
        .with_application_advancement(&live, |phase| {
            worth_execution::with_request_consult_observer_for_test(
                |_| {},
                || call(home.runtime(), &phase, &live, &principal),
            )
        })
        .expect("the real Home admits a fresh business scope");
    assert!(matches!(healthy, Ok(Outcome::RecoveredCommitted(_))));
    assert!(consults > 0);
    for stop_at in 1..=consults {
        let (cancel, stopped) = scope();
        let principal = identity::inspection_authentication::authenticate(
            home.runtime().runtime().installed_schema(),
            &stopped,
        );
        let (answer, reached) = home
            .runtime()
            .runtime()
            .with_application_advancement(&stopped, |phase| {
                worth_execution::with_request_consult_observer_for_test(
                    move |at| {
                        if at == stop_at {
                            cancel.cancel();
                        }
                    },
                    || call(home.runtime(), &phase, &stopped, &principal),
                )
            })
            .expect("the real Home admitted the original business scope before injection");
        assert!(
            reached >= stop_at,
            "the original path changed before consult {stop_at}"
        );
        assert!(
            !matches!(&answer, Ok(Outcome::RecoveredCommitted(_))),
            "business cancellation at consult {stop_at} of {consults} was lost: {answer:?}",
        );
        assert_cancelled(answer);
    }
    let (cancel, late) = scope();
    let principal = identity::inspection_authentication::authenticate(
        home.runtime().runtime().installed_schema(),
        &late,
    );
    let completed = home
        .runtime()
        .runtime()
        .with_application_advancement(&late, |phase| {
            let answer = call(home.runtime(), &phase, &late, &principal);
            cancel.cancel(); // The business result already exists; upkeep cannot replace it.
            answer
        })
        .expect("late cancellation preserves the admitted business result");
    let recorded = match completed {
        Ok(Outcome::RecoveredCommitted(receipt)) => receipt,
        other => panic!("post-business cancellation must preserve the original answer: {other:?}"),
    };
    let original = match &fresh {
        Outcome::Committed { receipt, .. } => receipt,
        other => panic!("the retained original must remain fresh: {other:?}"),
    };
    receipt_fields::assert_live_agrees(original, &recorded);
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
    assert_eq!(balance(home.runtime(), &live), 8);
    drop(fresh);
    drop(home);
}
