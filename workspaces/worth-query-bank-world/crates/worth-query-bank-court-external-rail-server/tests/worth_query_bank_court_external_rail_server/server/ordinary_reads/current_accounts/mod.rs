//! Genuine ordinary Source parity, incremental mutations and current-only custody.
use worth_query_bank_testkit_server::facade::ordinary_reads::current_accounts::*;
mod membership;
mod snapshots;

fn work(runtime: &bank_server::BankIdentityRuntime) -> (usize, usize, usize) {
    runtime.account_index_work_for_test()
}
fn incremental(before: (usize, usize, usize), after: (usize, usize, usize)) {
    assert_eq!(after.0, before.0, "a warm membership edit must not rebuild");
    assert!(
        after.1 > before.1 && after.2 > before.2,
        "actual changed occurrences reached update: {before:?}->{after:?}"
    );
}
macro_rules! mutate {
    ($fixture:expr, $actor:expr, $input:expr, $key:expr) => {{
        let scope = request_scope();
        let request = $fixture.world.runtime.request($actor, &scope);
        let operation = $fixture
            .world
            .runtime
            .mutate($input)
            .as_principal($actor)
            .controls(BankMutationControls::new(
                scope.clone(),
                bank_domain::proposals::BankIdempotencyKey::new($key).unwrap(),
            ))
            .retain_operation()
            .unwrap_or_else(|cause| panic!("actual owning mutation: {cause:?}"));
        complete(operation.advance(&request), |next| next.advance(&request))
            .close(&request)
            .unwrap_or_else(|(_, cause)| panic!("actual owning close: {cause:?}"));
    }};
}
use mutate;
fn complete<O, C, T>(
    mut advance: WorthQuerySourceOnlyAdvance<O, C, T>,
    mut step: impl FnMut(O) -> WorthQuerySourceOnlyAdvance<O, C, T>,
) -> C {
    for _ in 0..16 {
        advance = match advance {
            WorthQuerySourceOnlyAdvance::Progress(next) => step(next),
            WorthQuerySourceOnlyAdvance::Completed(done) => return done,
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQuerySourceOnlyAdvance::Terminal(_) => panic!("unexpected terminal"),
        };
    }
    panic!("bounded actual owning mutation");
}
#[test]
fn current_accounts_preserve_three_paths_dedup_scope_and_unopened_empty_roots() {
    let fixture = ordinary_read_world("managed-account-parity", 8);
    for principal in [
        OWNER, APPROVER, VIEWER, RECIPIENT, STRANGER, TELLER, AUDITOR,
    ] {
        let actor = fixture.authenticate(principal);
        let expected = ordinary(&fixture, &actor);
        let actual = managed(&fixture, &actor);
        assert_eq!(actual, expected);
        assert_eq!(
            actual
                .iter()
                .map(|row| row.id())
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            actual.len()
        );
        if principal == OWNER {
            let mut ids = vec![fixture.personal_account, fixture.business_account];
            ids.sort();
            assert_eq!(actual.iter().map(|row| row.id()).collect::<Vec<_>>(), ids);
        }
        if principal == STRANGER {
            assert!(actual.is_empty());
        }
        let warmed = work(&fixture.world.runtime);
        assert_eq!(managed(&fixture, &actor), actual);
        assert_eq!(
            work(&fixture.world.runtime).0,
            warmed.0,
            "repeat read uses the maintained root"
        );
    }
}
#[test]
fn current_accounts_keep_actual_result_limit_and_cancellation() {
    let fixture = over_budget_discovery_world("managed-account-limit", 160);
    let actor = fixture.authenticate();
    let result = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 159).unwrap())
        .execute();
    assert!(
        matches!(&result, Err(BankApplicationQueryDenial::ManagedIndex(denial)) if matches!(denial.cause(), WorthQueryManagedIndexDenialCause::QueryRead(cause) if cause.kind() == WorthQueryApplicationOneShotDenialKind::ResultLimitExceeded)),
        "actual result-limit denial: {:?}",
        result.as_ref().err()
    );
    let original = fixture
        .world
        .runtime
        .query(queries::accounts())
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 160).unwrap())
        .execute()
        .unwrap();
    let actual = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 160).unwrap())
        .execute()
        .unwrap();
    assert_eq!(actual.rows(), original.rows());
    let repeated = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&actor)
        .controls(BankReadControls::current(request_scope(), 160).unwrap())
        .execute()
        .unwrap();
    assert_eq!(repeated.rows(), original.rows());
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let result = fixture
        .world
        .runtime
        .query(queries::current_accounts())
        .as_principal(&actor)
        .controls(
            BankReadControls::current(WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 160)
                .unwrap(),
        )
        .execute();
    assert!(matches!(result, Err(BankApplicationQueryDenial::ExecutionRequest(
        worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(
            worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled
        )
    ))));
}
