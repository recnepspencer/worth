use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_relational::facade::history::CommitId;

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::installed_world;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope;
use crate::domain_computation::primary_graph::tests::inbound_admission::verifier::signed_envelope_for_seconds;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::output_lineage::invalidation::commit_touches::CommitTouchInterest;
use worth_query_execution::facade::domain_computation::WorthQueryInboundReceiptPosture;

#[test]
fn performed_completion_reaches_the_canonical_subscription_once_and_terminal_turns_over() {
    let world = installed_world();
    let dispatch = { world.commit_dispatch(51, "route-observed-notice") };
    let outbox = dispatch.dispatch_outbox().unwrap();
    let owner = (*(*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .source_owner_for_test())
    .invalidation_owner_for_test();
    let branch = &dispatch.commit_reference().branch_id;
    let before = owner.latest_position_for_test();
    let envelope = signed_envelope_for_seconds(outbox, [0xb1; 32], outbox.payload(), false, 1);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let after = owner.latest_position_for_test();
    let unwatched = BTreeSet::new();
    let batch = owner.touched_commits_after_for_test(
        branch,
        before,
        Some(CommitId(u64::MAX)),
        2,
        CommitTouchInterest::new_for_test((&unwatched).into(), true),
    );
    assert_eq!(
        (*batch.commits_for_test()).len(),
        1,
        "one performed completion reaches the canonical subscription"
    );
    assert_ne!(
        (*(*batch.commits_for_test())[0].commit_for_test()),
        dispatch.commit_reference().commit_id,
        "the subscription carries the completion commit, not the old dispatch",
    );
    assert_eq!((*batch.next_cursor_for_test()), after);
    assert!((*batch.caught_up_to_latest_for_test()));

    let replay = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert!(matches!(
        replay.posture(),
        WorthQueryInboundReceiptPosture::Performed
            | WorthQueryInboundReceiptPosture::AlreadyAccepted
            | WorthQueryInboundReceiptPosture::AlreadyCompleted
    ));
    assert_eq!(
        owner.latest_position_for_test(),
        after,
        "retry must not publish the completion twice",
    );

    std::thread::sleep(Duration::from_secs(2));
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&world.verifier, NonZeroUsize::new(1).unwrap(),)
            .unwrap()
            .reclaimed(),
        1,
        "delivered terminal no longer pins the accepted slot",
    );
    let next = { world.commit_dispatch(52, "after-route-turnover") };
    let next_outbox = next.dispatch_outbox().unwrap();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(
                &world.verifier,
                &signed_envelope(next_outbox, [0xb2; 32], next_outbox.payload(), false),
                &request,
            )
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
}
