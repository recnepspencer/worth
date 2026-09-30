//! A later ordinary commit cannot redirect an earlier dispatch's completion.

use super::fixture::installed_world;
use super::tests::{completion_records, owner_commits, product_commit};
use super::verifier::signed_envelope;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn accepted_completion_uses_original_incarnation_and_current_world_head() {
    let world = installed_world();
    let first = world.commit_dispatch(181, "first-dispatch");
    let second = world.commit_dispatch(182, "later-dispatch");
    assert_eq!(
        first.committed_product_publication().product_incarnation(),
        second.committed_product_publication().product_incarnation()
    );
    assert_ne!(
        first.committed_product_publication().composite_commit(),
        second.committed_product_publication().composite_commit()
    );
    let first_record = first.dispatch_outbox().unwrap();
    let envelope = signed_envelope(first_record, [0x61; 32], first_record.payload(), false);
    let head_before_completion = product_commit(&world);
    let commits_before_completion = owner_commits(&world);
    let receipt = world
        .application
        .receive_inbound_occurrence(
            &world.verifier,
            &envelope,
            &super::super::fixture::live_scope(),
        )
        .unwrap();
    assert_eq!(
        receipt.posture(),
        WorthQueryInboundReceiptPosture::Performed
    );
    let terminal = world
        .application
        .observe_inbound_terminal(&world.verifier, *first_record.correlation().bytes())
        .expect("the real owner exposes the one completed effect");
    assert_eq!(
        terminal.original_world_commit(),
        first.committed_product_publication().composite_commit()
    );
    assert_ne!(terminal.completion_world_commit(), &head_before_completion);
    assert_eq!(terminal.completion_world_commit(), &product_commit(&world));
    assert_eq!(completion_records(&world), 1);
    assert_eq!(owner_commits(&world), commits_before_completion + 1);
}
