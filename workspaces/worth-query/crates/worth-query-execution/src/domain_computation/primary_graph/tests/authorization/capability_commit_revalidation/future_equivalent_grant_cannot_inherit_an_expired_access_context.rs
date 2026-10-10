//! future equivalent grant cannot inherit an expired access context.
use super::*;

#[test]
fn future_equivalent_grant_cannot_inherit_an_expired_access_context() {
    let world = installed_capability_replacement_world();
    world.authorization_time.script([
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
        time(100),
    ]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 56);
    world.authorization_time.script([time(120)]);

    assert_readmission_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(56, 56),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    ));
}
