use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::time::Duration;

use worth_relational::facade::history::CommitId;

use super::fixture::installed_world;
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::primary_graph::output_lineage::invalidation::CommitTouchInterest;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn performed_completion_reaches_the_canonical_subscription_once_and_terminal_turns_over() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(51, "route-observed-notice");
    let outbox = dispatch.dispatch_outbox().unwrap();
    let owner = &world
        .application
        .primary_provider
        .graph
        .source_owner
        .invalidation_owner;
    let branch = &dispatch.commit_reference().branch_id;
    let before = owner.latest_position();
    let envelope = signed_envelope_for_seconds(outbox, [0xb1; 32], outbox.payload(), false, 1);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let after = owner.latest_position();
    let unwatched = BTreeSet::new();
    let batch = owner
        .touched_commits_after(
            branch,
            before,
            Some(CommitId(u64::MAX)),
            2,
            CommitTouchInterest {
                entities: &unwatched,
                every_commit: true,
            },
        )
        .unwrap();
    assert_eq!(
        batch.commits.len(),
        1,
        "one performed completion reaches the canonical subscription"
    );
    assert_ne!(
        batch.commits[0].commit,
        dispatch.commit_reference().commit_id,
        "the subscription carries the completion commit, not the old dispatch",
    );
    assert_eq!(batch.next_cursor, after);
    assert!(batch.caught_up_to_latest);

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
        owner.latest_position(),
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
    let next = world.commit_dispatch(52, "after-route-turnover");
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
