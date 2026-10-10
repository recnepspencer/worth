//! response loss and interleaving preserve one preimage and outbox bundle.
use super::*;

#[test]
fn response_loss_and_interleaving_preserve_one_preimage_and_outbox_bundle() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
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
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            resolved_account(&(phase).test_host_request(), &world, "unrelated", &request)
        })
        .expect("the fixture read owns its installed host call");
    let selected = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let commit_count = || {
        world
            .application
            .primary_provider
            .graph
            .with_runtime(request_execution, |runtime| {
                runtime.history().immutable_commit_count()
            })
    };
    let baseline = commit_count();
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retained_status_program(
                &(phase).test_host_request(),
                &world,
                &principal,
                &account,
                &request,
                "frozen",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    let retry = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retained_status_program(
                &(phase).test_host_request(),
                &world,
                &principal,
                &account,
                &request,
                "frozen",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");
    let interleaved = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retained_status_program(
                &(phase).test_host_request(),
                &world,
                &principal,
                &unrelated,
                &request,
                "changed-between",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");

    let readmitted_interleave = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retained_status_program(
                &(phase).test_host_request(),
                &world,
                &principal,
                &unrelated,
                &request,
                "changed-between",
                RetentionMutationBreadth::Narrow,
            )
        })
        .expect("the fixture read owns its installed host call");

    world.faults.lose_next_commit_response();
    let outcome = world.application.compare_and_commit_application(
        first,
        idempotency(81, 82),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(original) = outcome else {
        panic!("response-loss recovery must return the authoritative commit: {outcome:?}");
    };
    let after_original = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_ne!(after_original.selected_commit(), selected.selected_commit());
    assert_eq!(commit_count(), baseline + 1);
    let outcome = world.application.compare_and_commit_application(
        interleaved,
        idempotency(83, 84),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(interleave_receipt) = outcome else {
        panic!("independent retained decision must commit: {outcome:?}");
    };
    assert_eq!(commit_count(), baseline + 2);
    assert_retained_status(&interleave_receipt, "unrelated");

    assert!(matches!(
        world.application.compare_and_commit_application(
            readmitted_interleave,
            idempotency(83, 84),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation
        ),
        WorthQueryApplicationCommitOutcome::AlreadyCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(81, 82),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("later equivalent resolution must reopen the original evidence");
    };

    assert!(recovered.is_same_authoritative_commit(&original));
    assert_eq!(recovered.mutation_work(), original.mutation_work());
    assert_eq!(recovered.dispatch_outbox(), original.dispatch_outbox());
    assert_eq!(recovered.retained_preimage(), original.retained_preimage());
    assert_eq!(original.terminal().attempt_resources_released(), Some(true));
    assert_eq!(recovered.terminal().attempt_resources_released(), None);
    assert!(recovered.dispatch_outbox().is_some());
    assert_retained_status(&recovered, "open");
    assert_eq!(commit_count(), baseline + 2);
}
