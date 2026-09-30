//! Receipt posture and retained maintenance work are separate facts.

use super::fixture::installed_world;
use super::verifier::signed_envelope;
use crate::domain_computation::primary_graph::WorthQueryInboundReceiptPosture;

#[test]
fn accepted_custody_cues_maintenance_but_terminal_lookup_does_not() {
    let world = installed_world();
    let receipt = world.commit_dispatch(118, "maintenance-cue");
    let outbox = receipt.dispatch_outbox().unwrap();
    let request = super::super::fixture::live_scope();
    let first = signed_envelope(outbox, [0xB2; 32], outbox.payload(), false);
    let accepted = world
        .application
        .receive_inbound_occurrence(&world.verifier, &first, &request)
        .unwrap();
    assert_eq!(
        accepted.posture(),
        WorthQueryInboundReceiptPosture::Performed
    );
    assert!(accepted.requires_maintenance_cue());

    let later = signed_envelope(outbox, [0xB3; 32], outbox.payload(), false);
    let completed = world
        .application
        .receive_inbound_occurrence(&world.verifier, &later, &request)
        .unwrap();
    assert_eq!(
        completed.posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted
    );
    assert!(!completed.requires_maintenance_cue());
}
