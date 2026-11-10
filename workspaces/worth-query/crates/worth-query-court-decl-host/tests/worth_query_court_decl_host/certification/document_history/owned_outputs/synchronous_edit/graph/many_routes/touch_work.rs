//! Touch admission projects one part per writer, so its work follows the starts.
use super::*;
use worth_relational::facade::mvcc::ValidatedRelationalProposal;

/// The validated intents every touch projection of one wide edit examines.
fn projected(routes: u64) -> u64 {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    seed(&host, &request, routes);
    let (outcome, examined) = ValidatedRelationalProposal::observe_touch_projection_work(|| {
        fan_out(&host, &request, routes, routes as usize + 1)
    });
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the wide edit performs"
    );
    examined
}

#[test]
fn touch_projection_work_follows_the_starts_not_their_square() {
    let (small, large) = (projected(16), projected(32));
    assert!(
        small > 0,
        "the edit's writers project their touches on the calling thread"
    );
    // 17 then 33 starts. One part per writer about doubles the work; projecting
    // every part for every writer about quadruples it.
    assert!(
        large * 2 <= small * 5,
        "projected intents grew from {small} to {large}"
    );
}
