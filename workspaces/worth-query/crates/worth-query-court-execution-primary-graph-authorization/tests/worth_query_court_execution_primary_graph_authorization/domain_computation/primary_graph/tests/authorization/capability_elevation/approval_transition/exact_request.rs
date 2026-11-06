//! The exact approval transition commits both derived effects.
use super::*;
#[test]
fn exact_request_approval_reobserves_lifecycle_and_commits_two_derived_effects() {
    let (world, request, requested) = requested_world(CapabilityElevationScenario::Active);
    let approver = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated(&host_custody, &world, "bob", &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            materialize_exact_approval(&host_custody, &world, &request, requested)
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryElevationApprovalOutcome::Approved(approved) = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(172, 172))
    else {
        panic!("the exact approval transition must commit");
    };
    assert_approved(&approved, &approver);
}
