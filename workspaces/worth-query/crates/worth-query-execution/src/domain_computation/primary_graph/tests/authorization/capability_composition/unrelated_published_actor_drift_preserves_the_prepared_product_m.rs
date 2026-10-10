//! unrelated published actor drift preserves the prepared product mutation.
use super::*;

#[test]
fn unrelated_published_actor_drift_preserves_the_prepared_product_mutation() {
    let world = installed_composed_capability_world(CapabilityCompositionScenario::Lawful);
    world.authorization_time.script(vec![time(100); 16]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            composed_program(&phase, &world, &principal, &request)
        })
        .expect("the fixture read owns its installed host call");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            add_action_policy_relation(
                &phase,
                &world,
                CapabilityRequestActor::reference(),
                "late-unrelated-request-actor",
                "other-request",
            )
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            add_action_policy_relation(
                &phase,
                &world,
                CapabilityPriorActor::reference(),
                "late-unrelated-prior-actor",
                "other-prior",
            )
        })
        .expect("the fixture read owns its installed host call");

    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(76, 76),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "unrelated actor edges must preserve current authorization and decision: {outcome:?}"
    );
}
