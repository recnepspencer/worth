use super::*;

#[test]
fn unwind_after_performed_publication_preserves_one_exact_retriable_record() {
    let world = installed_authorization_world(true);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let before = selected.product().selected_commit().clone();
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program_with_emit(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "post-performed-unwind",
                Some("post-performed-effect"),
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            admitted_program(
                &phase,
                &world,
                &principal,
                &account,
                &request,
                "post-performed-unwind",
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.panic_next_pending_application_publication();
    let WorthQueryApplicationCommitOutcome::Committed(first_receipt) =
        world.application.compare_and_commit_application(
            first,
            idempotency(198, 199),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the public commit journey must rediscover and finish the exact retained record");
    };
    assert_eq!(
        world
            .faults
            .panicked_pending_publication_consumption_count(),
        1,
        "the post-Performed unwind must occur exactly once"
    );
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0,
        "the internal exact retry must retire the recovered slot"
    );
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_ne!(
                world
                    .selected_product(&(phase).test_host_request(),)
                    .product()
                    .selected_commit(),
                &before,
                "World must already expose the performed successor"
            )
        })
        .expect("the fixture read owns its installed host call");

    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(198, 199),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the equivalent retry must recover the exact performed publication");
    };
    assert!(recovered.is_same_authoritative_commit(&first_receipt));
    assert_eq!(recovered.emitted_effect_count(), 1);
    assert_eq!(
        world
            .application
            .primary_provider
            .pending_application_publication_count_for_test(),
        0,
        "terminal recovery must disarm and retire the exact slot"
    );
    let emissions = world
        .application
        .primary_provider
        .committed_application_emissions(recovered.committed_product_publication());
    assert_eq!(emissions.len(), 1);
    assert_eq!(
        emissions[0].payload::<String>().map(String::as_str),
        Some("post-performed-effect")
    );
}
