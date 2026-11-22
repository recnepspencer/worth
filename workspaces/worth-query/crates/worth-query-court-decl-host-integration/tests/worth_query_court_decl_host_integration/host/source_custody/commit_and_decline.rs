//! Generic helpers commit, read and decline through custody exactly as the
//! direct Host calls do: the drifted key is the native owner's own refusal.
use super::*;

#[test]
fn bundled_helpers_commit_read_and_decline_a_drifted_key() {
    let clock = WorthQueryAdmissionClock::system();
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let limits = WorthQueryApplicationLimits::bounded_serial(clock.clone())
        .install()
        .unwrap();
    let home = open_home(&execution, limits)
        .unwrap_or_else(|refused| panic!("the bounded Home opens: {refused:?}"));
    let scope = scope(&clock);
    let principal = local_principal(&home, &clock, "subject", Duration::from_secs(3600))
        .authenticate_local(&scope)
        .unwrap();
    {
        let request = home.runtime().request(&principal, &scope);
        assert_eq!(read(&request, account()), Some(vec![7]));
        let first = capture(&request, set_balance(8), 0x4355_5354_u64);
        let step = Custody::Described(first).resume(home.runtime(), &request);
        assert_eq!(settled(step), "committed 8");
        assert_eq!(read(&request, account()), Some(vec![8]));

        let drifted = capture(&request, set_balance(9), 0x4355_5354_u64);
        let step = Custody::Described(drifted).resume(home.runtime(), &request);
        assert_eq!(settled(step), "declined IdempotencyIntentDrift");
        assert_eq!(read(&request, account()), Some(vec![8]));
    }
    drop(principal);
    close_home(home, &execution);
}
