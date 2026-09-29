use std::num::NonZeroUsize;

use super::fixture::installed_world;
use super::verifier::signed_envelope;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn retired_original_incarnation_rebuilds_from_world_and_canonical_relational_patch() {
    let world = installed_world();
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().fork_signal())
        .create()
        .expect("forked application product must publish");
    let (correlation, original_world_commit) = {
        let dispatch = world.commit_dispatch_on(branch, 71, "notice-retired-terminal");
        let record = dispatch.dispatch_outbox().unwrap();
        let correlation = *record.correlation();
        let envelope = signed_envelope(record, [0xb1; 32], record.payload(), false);
        let request = super::super::fixture::live_scope();
        assert_eq!(
            world
                .application
                .receive_inbound_occurrence(&world.verifier, &envelope, &request,)
                .unwrap()
                .posture(),
            WorthQueryInboundReceiptPosture::Performed
        );
        let original = dispatch
            .committed_product_publication()
            .composite_commit()
            .clone();
        (correlation, original)
    };
    assert!(matches!(
        world.application.on_branch(branch).close(),
        Err(crate::domain_computation::primary_graph::WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(_))
    ));
    assert!(world
        .application
        .product_runtime()
        .admit_product_occurrence(branch.occurrence())
        .is_err());

    let provider = &world.application.primary_provider;
    provider.destroy_completed_inbound_index_for_test();
    assert!(provider.lookup_completed_inbound(&correlation).is_err());
    let complete = world
        .application
        .rebuild_completed_inbound_index(
            NonZeroUsize::new(32).unwrap(),
            NonZeroUsize::new(32).unwrap(),
            NonZeroUsize::new(32).unwrap(),
        )
        .expect("World-paired canonical commit patch survives branch retirement");
    assert!(complete);
    let rebuilt = provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .unwrap();
    assert_eq!(rebuilt.original_world_commit(), &original_world_commit);
    assert_ne!(
        rebuilt.original_world_commit(),
        rebuilt.completion_world_commit()
    );
}
