use worth_runtime_world::facade::CompositeComponentChangePosture;

use super::WorthQueryInboundPublicationOutcome;
use crate::domain_computation::application_aftermath::{
    WorthQueryAcceptedInboundOccurrence, WorthQueryInboundOccurrenceClaims,
};
use crate::domain_computation::primary_graph::tests::{
    fixture::live_scope, recoverable_commit_support::two_recoverable_application_commits,
};
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;

#[test]
fn accepted_completion_publishes_one_real_relational_effect_on_the_original_incarnation() {
    // The second ordinary commit advances the current head after the first
    // dispatch. This test enters after authentication and exact acceptance;
    // the Bank process court covers those earlier boundaries.
    let (world, first, second) = two_recoverable_application_commits(181, 182);
    let owner = world
        .application
        .observe_committed_dispatch_outbox(&first)
        .expect("the original performed dispatch remains readable")
        .expect("the first commit contains a real dispatch outbox row");
    let record = owner.record();
    let original_incarnation = owner.committed_product_publication().product_incarnation();
    assert_eq!(
        original_incarnation,
        second.committed_product_publication().product_incarnation()
    );
    assert_ne!(
        owner.committed_product_publication().composite_commit(),
        second.committed_product_publication().composite_commit()
    );
    let accepted = WorthQueryAcceptedInboundOccurrence::seal_for_world_test(
        WorthQueryInboundOccurrenceClaims {
            audience: "world-substrate-test".to_owned(),
            source_identity: "world-substrate-source".to_owned(),
            key_epoch: 1,
            message_identity: [0x61; 32],
            signed_meaning_digest: [0x71; 32],
            issued_at_unix_seconds: 1,
            expires_at_unix_seconds: 2,
            protocol_identity: record.protocol_identity().clone(),
            protocol_version: record.protocol_version(),
            correlation_family: record.correlation_family().as_str().to_owned(),
            correlation_token: *record.correlation().bytes(),
            payload: record.payload().to_vec(),
        },
        owner,
    );
    let outcome = world
        .application
        .publish_inbound_completion(accepted, &live_scope());
    let WorthQueryInboundPublicationOutcome::Performed(performed) = outcome else {
        panic!("one accepted completion should publish at the current head");
    };
    assert_eq!(performed.original_incarnation(), original_incarnation);
    let completion =
        WorthQueryCommittedProductPublication::from_receipt(performed.publication().clone());
    assert_eq!(completion.product_incarnation(), original_incarnation);
    assert_ne!(
        completion.composite_commit(),
        second.committed_product_publication().composite_commit()
    );
    assert_ne!(completion.relational_commit(), second.commit_reference());
    assert_eq!(
        completion.relational_posture(),
        CompositeComponentChangePosture::Published
    );
    assert_eq!(
        completion.signal_posture(),
        CompositeComponentChangePosture::RetainExact
    );
}
