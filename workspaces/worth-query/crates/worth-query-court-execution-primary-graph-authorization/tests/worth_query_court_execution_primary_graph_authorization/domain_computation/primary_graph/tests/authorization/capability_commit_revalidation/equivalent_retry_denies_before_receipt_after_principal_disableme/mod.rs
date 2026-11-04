//! equivalent retry denies before receipt after principal disablement.
use crate::domain_computation::primary_graph::tests::authorization::capability_commit_revalidation::*;

#[test]
fn equivalent_retry_denies_before_receipt_after_principal_disablement() {
    let world = current_retry_world();
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request_for_test(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (first, retry) = world
        .application
        .with_host_advancement(|phase| equivalent_programs(&phase, &world, &principal, &request))
        .expect("the fixture read owns its installed host call");
    commit_first(&world, first, 52);
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request_for_test();
            disable_mapping(
                &host_custody.borrowed_request_for_test(),
                &world,
                principal.mapping_entity_id_for_test(),
            )
        })
        .expect("the fixture read owns its installed host call");

    assert_readmission_denial(world.application.compare_and_commit_application(
        retry,
        idempotency(52, 52),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    ));
}
