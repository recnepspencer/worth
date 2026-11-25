//! Bounded composite-history retention survives repeated commits across
//! graceful close, Directory Home reopen, and further commits.
use super::*;
use std::sync::{atomic::AtomicUsize, Arc};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::{ApplicationBuildIdentity, WorthQueryBoundedApplicationLimits},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    runtime::RuntimeWorldHistoryBudgetInstallation,
};

#[test]
fn bounded_history_survives_commits_graceful_close_reopen_and_more_commits() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let location = fixture::location();
    let build = ApplicationBuildIdentity::parse("33333333-3333-4333-8333-333333333333").unwrap();
    let history = RuntimeWorldHistoryBudgetInstallation {
        retained_composite_commits: 4,
        ..WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS.history
    };
    let open_bounded = || {
        let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
            .with_history(history)
            .install()
            .unwrap();
        let request = fixture::request_with_limits(
            location.clone(),
            build,
            Arc::new(AtomicUsize::new(0)),
            limits,
            |_, graph, schema| {
                fixture::schema::seed(graph, schema)?;
                Ok::<(), WorthQueryPrimaryGraphInstallationDenial>(())
            },
        );
        with_bootstrap_advancement_from_request(execution, |phase| {
            request.open_in_advancement(&phase)
        })
        .expect("bounded Home bootstrap executes")
        .unwrap_or_else(|refused| panic!("bounded Directory Home opens: {refused:?}"))
    };
    let scope = scope(&clock);

    // Opening 1: 12 commits under retained_composite_commits = 4, then graceful close.
    let home = open_bounded();
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    for step in 1_u64..=12 {
        let value = 100 + step;
        let request = home.runtime().request(&principal, &scope);
        let described = capture(&request, set_balance(value), 0xD471_0000_u64 + step);
        assert_eq!(
            settled(Custody::Described(described).resume(home.runtime(), &request)),
            format!("committed {value}")
        );
    }
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![112]));
    drop(principal);
    close_home(home, &execution);

    // Opening 2: reopen same Directory Home under retained_composite_commits = 4,
    // perform 12 more commits, verify balance == 124, then graceful close.
    let home = open_bounded();
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![112]));
    for step in 13_u64..=24 {
        let value = 100 + step;
        let request = home.runtime().request(&principal, &scope);
        let described = capture(&request, set_balance(value), 0xD471_0000_u64 + step);
        assert_eq!(
            settled(Custody::Described(described).resume(home.runtime(), &request)),
            format!("committed {value}")
        );
    }
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![124]));
    drop(principal);
    close_home(home, &execution);

    // Opening 3: reopen once more and confirm final persisted balance == 124.
    let home = open_bounded();
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![124]));
    drop(principal);
    close_home(home, &execution);
}
