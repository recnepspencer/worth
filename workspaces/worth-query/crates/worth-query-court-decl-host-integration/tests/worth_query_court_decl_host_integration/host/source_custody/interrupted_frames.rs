//! Custody outlives each request frame: a cancelled retention and an expired
//! principal stay pending with their native refusals, fresh frames finish
//! them, and a full bounded history declines and closes its terminal.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource,
        WorthQueryRequestScope,
    },
    application_entry::WorthQuerySourceOnlyDenial,
    application_installation::WorthQueryBoundedApplicationLimits,
    runtime::RuntimeWorldHistoryBudgetInstallation,
};

type Home = super::super::application_startup::Home;
type Principal = WorthQueryAuthenticatedExternalPrincipal<Schema>;

#[test]
fn custody_survives_interrupted_frames_and_declines_once_history_is_full() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    // The seed and the two commits below fill the retained history.
    let history = RuntimeWorldHistoryBudgetInstallation {
        retained_composite_commits: 3,
        ..WorthQueryBoundedApplicationLimits::DEFAULT_BUDGETS.history
    };
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .with_history(history)
        .install()
        .unwrap();
    let home = open_home(&execution, limits)
        .unwrap_or_else(|refused| panic!("the bounded Home opens: {refused:?}"));
    let scope = scope(&clock);
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    let fresh_frame = |custody: Custody| {
        let request = home.runtime().request(&principal, &scope);
        settled(custody.resume(home.runtime(), &request))
    };

    let custody = cancelled_before_retention(&home, &principal, &clock, set_balance(5));
    assert_eq!(fresh_frame(custody), "committed 5");

    let brief = Duration::from_millis(50);
    let expiring = local_principal(&home, &clock, "subject", brief)
        .authenticate_local(&scope)
        .unwrap();
    std::thread::sleep(brief + Duration::from_millis(20));
    let custody = refused_after_expiry(&home, &expiring, &scope, set_balance(6));
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![5]));
    assert_eq!(fresh_frame(custody), "committed 6");

    let declined = {
        let request = home.runtime().request(&principal, &scope);
        let described = capture(&request, set_balance(9), 0x4652_4d35_u64);
        settled(Custody::Described(described).resume(home.runtime(), &request))
    };
    assert!(
        declined.contains("ProductPreparationRefused") && declined.contains("CapacityExhausted"),
        "the full history refuses before preparation: {declined}"
    );
    assert_eq!(read_balance(&home, &principal, &scope), Ok(vec![6]));
    drop((principal, expiring));
    close_home(home, &execution);
}

/// A frame captures `command`, then is cancelled before durable retention.
fn cancelled_before_retention(
    home: &Home,
    principal: &Principal,
    clock: &WorthQueryAdmissionClock,
    command: Balance,
) -> Custody {
    let cancellation = WorthQueryCancellationSource::new();
    let cancelled = WorthQueryRequestScope::new(None, cancellation.token(), clock.clone());
    let request = home.runtime().request(principal, &cancelled);
    let described = capture(&request, command, 0x4652_4d31_u64);
    cancellation.cancel();
    let (custody, cause) = pending(Custody::Described(described).resume(home.runtime(), &request));
    assert!(
        matches!(cause, WorthQuerySourceCustodyPending::Retention(_)),
        "{cause:?}"
    );
    assert!(matches!(custody, Custody::Described(_)));
    custody
}

/// A frame retains `command`, whose advance refuses the expired principal.
fn refused_after_expiry(
    home: &Home,
    expired: &Principal,
    scope: &WorthQueryRequestScope,
    command: Balance,
) -> Custody {
    let request = home.runtime().request(expired, scope);
    let described = capture(&request, command, 0x4652_4d33_u64);
    let (custody, cause) = pending(Custody::Described(described).resume(home.runtime(), &request));
    assert!(
        matches!(
            cause,
            WorthQuerySourceCustodyPending::Refused(WorthQuerySourceOnlyDenial::Mutation(_))
        ),
        "{cause:?}"
    );
    assert!(
        format!("{cause:?}").contains("ExpiredAuthentication"),
        "{cause:?}"
    );
    assert!(matches!(custody, Custody::Retained(_)));
    custody
}
