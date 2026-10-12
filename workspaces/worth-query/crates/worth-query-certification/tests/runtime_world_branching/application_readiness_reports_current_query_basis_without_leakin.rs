//! application readiness reports current query basis without leaking a lease.
use super::*;

#[test]
fn application_readiness_reports_current_query_basis_without_leaking_a_lease() {
    let world = CourtroomWorld::publish("ready");
    let branch = world.application.current_world();
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();

    let readiness = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the published product should be selectable")
                .inspect_application_readiness()
                .expect("the published application basis should be inspectable")
        })
        .expect("installed host call opens");

    assert_eq!(
        readiness.schema_binding(),
        &world.application.installed_schema().binding_identity()
    );
    assert!(readiness
        .basis_token()
        .starts_with("basis:query-primary-graph-v2:"));
    let repeated = world
        .application
        .with_host_advancement(|host_phase| {
            world
                .application
                .on_branch(branch)
                .select(&host_phase)
                .expect("the published product should remain selectable")
                .inspect_application_readiness()
                .expect("repeated readiness inspection should remain available")
        })
        .expect("installed host call opens");
    assert_eq!(
        repeated.basis_token(),
        readiness.basis_token(),
        "readiness inspection must not manufacture a new optimistic basis"
    );
    assert_eq!(
        world
            .application
            .application_basis_token(readiness.basis_identity())
            .expect("the observed query basis should render through its owning runtime"),
        readiness.basis_token(),
        "a result must retain its exact observed basis instead of rediscovering head"
    );
    let after = observer.observe();
    assert_eq!(after.active(), before.active());
    assert_eq!(after.acquisitions(), before.acquisitions() + 2);
}
