use std::num::NonZeroUsize;
use std::time::Duration;

use super::fixture::installed_world;
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::primary_graph::conditional_operation::install_test_whole_graph_route;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn nonempty_whole_graph_route_receives_completion_and_terminal_turns_over() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(51, "route-observed-notice");
    let outbox = dispatch.dispatch_outbox().unwrap();
    {
        let mut registry = world
            .application
            .conditional_operations
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        install_test_whole_graph_route(&mut registry);
        assert_eq!(registry.len(), 1);
        registry.synchronize_commit_routes(&world.application);
    }
    let before = world
        .application
        .primary_provider
        .conditional_commit_sequence();
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
    let after = world
        .application
        .primary_provider
        .conditional_commit_sequence();
    assert_eq!(
        after,
        before + 1,
        "one performed completion enters the journal"
    );
    let batch = world
        .application
        .primary_provider
        .conditional_commits_after_records(
            &dispatch.commit_reference().branch_id,
            Some(worth_relational::facade::history::CommitId(u64::MAX)),
            before,
            2,
            std::iter::empty(),
            true,
            None,
        )
        .unwrap();
    assert_eq!(batch.commits.len(), 1);
    assert_ne!(
        batch.commits[0].1,
        dispatch.commit_reference().commit_id,
        "the route receives the completion commit, not the old dispatch",
    );

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
        world
            .application
            .primary_provider
            .conditional_commit_sequence(),
        after,
        "retry must not hand off the completion twice",
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
