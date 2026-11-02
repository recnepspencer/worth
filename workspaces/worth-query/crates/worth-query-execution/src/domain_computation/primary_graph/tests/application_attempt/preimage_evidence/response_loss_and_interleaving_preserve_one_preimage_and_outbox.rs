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
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request)
        })
        .expect("the fixture read owns its installed host call");
    let unrelated = world
        .application
        .with_host_advancement(|phase| {
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
    let WorthQueryApplicationCommitOutcome::RecoveredCommitted(original) = outcome else {
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
        WorthQueryApplicationCommitOutcome::RecoveredCommitted(_)
    ));
    assert_eq!(commit_count(), baseline + 2);
    let WorthQueryApplicationCommitOutcome::RecoveredCommitted(recovered) =
        world.application.compare_and_commit_application(
            retry,
            idempotency(81, 82),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("later equivalent resolution must reopen the original evidence");
    };

    assert_eq!(recovered, original);
    // Recorded data cannot supply live preimage authority. Inspect the actual
    // already-retained owner custody independently, without minting a receipt.
    assert!(recovered.world_journal_position().is_none());
    assert!(recovered.dispatch_outbox().is_some());
    assert_eq!(
        world.application.provider_session_resource_count_for_test(),
        0
    );
    assert_eq!(commit_count(), baseline + 2);
    let commit = original.commit_reference().clone();
    let owner_evidence = world
        .application
        .primary_provider
        .observe_completed_application(&commit)
        .expect("the actual performed source still owns its preimage");
    let retained = owner_evidence
        .commit_evidence()
        .retained_preimage()
        .unwrap();
    assert!(!format!("{retained:?}").contains("open"));
    assert_eq!(
        retained
            .field_for(AccountStatus::reference())
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from("open")
        )
    );
    let original_meaning = owner_evidence
        .commit_evidence()
        .original_authority()
        .clone();
    let original_preimage = owner_evidence
        .commit_evidence()
        .retained_preimage()
        .cloned();
    drop(owner_evidence);
    drop(original);
    drop(recovered);
    let occurrence = world
        .application
        .primary_provider
        .observe_completed_application(&commit)
        .expect("response loss and receipt disposal preserve the performed source");
    assert_eq!(
        occurrence.commit_evidence().original_authority(),
        &original_meaning
    );
    assert_eq!(
        occurrence.commit_evidence().retained_preimage(),
        original_preimage.as_ref()
    );
}
