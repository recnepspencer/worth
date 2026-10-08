use super::*;

#[test]
fn unwind_after_performed_publication_preserves_one_exact_retriable_record() {
    let world = installed_authorization_world(true);
    let selected = world.selected_product();
    let _live = world
        .application
        .primary_provider
        .observe_application_commit_causality(selected.product());
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let before = selected.product().selected_commit().clone();
    let first = admitted_program_with_emit(
        &world,
        &principal,
        &account,
        &request,
        "post-performed-unwind",
        Some("post-performed-effect"),
    );
    let retry = admitted_program(
        &world,
        &principal,
        &account,
        &request,
        "post-performed-unwind",
    );

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
    assert_ne!(
        world.selected_product().product().selected_commit(),
        &before,
        "World must already expose the performed successor"
    );

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
