//! An equal inbound contract cannot replace the operation co-committed with an outbox.

use std::sync::Arc;

use super::fixture::installed_world;
use super::schema::OtherNotifyOperation;
use super::verifier::{signed_envelope, TestVerifier};
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundReceiptPosture,
};

#[test]
fn equal_contract_on_another_operation_cannot_claim_the_original_dispatch() {
    let world = installed_world();
    let receipt = world.commit_other_dispatch(117, "second-operation-original");
    let outbox = receipt
        .dispatch_outbox()
        .expect("the second operation really committed");
    assert_eq!(outbox.operation_slot(), Some("OtherNotifyOperation"));
    let envelope = signed_envelope(outbox, [0xB1; 32], outbox.payload(), false);
    let request = super::super::fixture::live_scope();

    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request),
        Err(WorthQueryInboundAdmissionDenial::UnsupportedOutbox)
    ));
    assert!(world
        .application
        .primary_provider
        .lookup_completed_inbound(outbox.correlation())
        .unwrap()
        .is_none());

    let operation = world
        .application
        .installed_schema()
        .installed_operation(OtherNotifyOperation::reference())
        .unwrap();
    let correct = world
        .application
        .install_inbound_occurrence_verifier(&operation, Arc::new(TestVerifier))
        .unwrap();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&correct, &envelope, &request)
            .expect("the original operation's installed route may complete it")
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
}
