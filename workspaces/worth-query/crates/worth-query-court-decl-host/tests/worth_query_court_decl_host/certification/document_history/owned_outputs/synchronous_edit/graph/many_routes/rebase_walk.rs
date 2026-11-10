//! Writers of one coupled commit share one walk of its patches after the commit.
use super::*;

/// The coupled commit patches every post-commit rebase of one wide edit walks.
fn walked(routes: u64) -> u64 {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    seed(&host, &request, routes);
    let before = host.coupled_commit_patches_walked_on_this_thread_for_test();
    let outcome = fan_out(&host, &request, routes, routes as usize + 1);
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the wide edit performs"
    );
    host.coupled_commit_patches_walked_on_this_thread_for_test() - before
}

#[test]
fn the_rebase_walks_a_coupled_commit_once_not_once_per_writer() {
    let (small, large) = (walked(16), walked(32));
    assert!(small > 0, "the writers' rebases see the coupled commit");
    // 17 then 33 writers, each adding its own patches. One walk per commit about
    // doubles the patches walked; one walk per writer about quadruples them.
    assert!(
        large * 2 <= small * 5,
        "patches walked grew from {small} to {large}"
    );
}
