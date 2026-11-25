//! Reopen verification helpers for early & post-Native generated materialization recovery.
use super::*;
use worth_store_testkit_application_oracle::facade::host_observation;

pub fn verify_suspension_reopen(
    location: &Path,
    skip_appends: usize,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let reopened = host_observation::opening::open_request(home_request(location), None, execution);
    let (reopen_scope, _) = scope();
    let reopen_principal = authenticate(&reopened, &reopen_scope);
    if skip_appends == 0 {
        assert_eq!(
            recovery_rows(&reopened),
            0,
            "early pre-Native suspension original must release on reopen"
        );
        let reopen_source = observe_source(&reopened, &reopen_principal, &reopen_scope);
        let reopen_branch = current_branch(&reopened);
        let rt = reopened.runtime().runtime();
        let selected = rt
            .with_host_advancement(|phase| rt.on_branch(reopen_branch).select(&phase).unwrap())
            .unwrap();
        let suspended = selected
            .suspend_current_generated_output::<OutputProducer>(&reopen_scope, reopen_source)
            .expect(
                "suspension succeeds cleanly after reopening the early-interrupted Directory Home",
            );
        drop(suspended);
    } else {
        assert_eq!(
            recovery_rows(&reopened),
            1,
            "post-Native suspension original must survive in World's recovery catalogue on reopen"
        );
        assert_eq!(release_recovered_originals(&reopened), 1);
        assert_eq!(
            recovery_rows(&reopened),
            0,
            "releasing the recovered post-Native suspension original resolves World's catalogue"
        );
        let reopen_source = observe_source(&reopened, &reopen_principal, &reopen_scope);
        drop(reopen_source);
    }
    drop(reopen_principal);
    let closed = reopened.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
    let reopened_again =
        host_observation::opening::open_request(home_request(location), None, execution);
    assert_eq!(recovery_rows(&reopened_again), 0);
    let closed = reopened_again.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}

pub fn verify_restoration_reopen(
    location: &Path,
    skip_appends: usize,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let reopened = host_observation::opening::open_request(home_request(location), None, execution);
    let (reopen_scope, _) = scope();
    let reopen_principal = authenticate(&reopened, &reopen_scope);
    if skip_appends == 0 {
        assert_eq!(
            recovery_rows(&reopened),
            0,
            "early pre-Native restoration original must release on reopen"
        );
    } else {
        assert_eq!(
            recovery_rows(&reopened),
            1,
            "post-Native restoration original must survive in World's recovery catalogue on reopen"
        );
        assert_eq!(release_recovered_originals(&reopened), 1);
        assert_eq!(
            recovery_rows(&reopened),
            0,
            "releasing the recovered post-Native restoration original resolves World's catalogue"
        );
    }
    // World's committed head is still the suspended commit in both cases, so
    // ordinary reads MUST continue refusing with BranchMaterializationSuspended.
    let denied = observe_source_result(&reopened, &reopen_principal, &reopen_scope)
        .expect_err("ordinary read must still refuse a suspended branch after reopen");
    assert!(
        format!("{denied:?}").contains("BranchMaterializationSuspended"),
        "ordinary read denial must remain BranchMaterializationSuspended: {denied:?}"
    );
    drop(reopen_principal);
    let closed = reopened.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
    let reopened_again =
        host_observation::opening::open_request(home_request(location), None, execution);
    assert_eq!(recovery_rows(&reopened_again), 0);
    let closed = reopened_again.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}
