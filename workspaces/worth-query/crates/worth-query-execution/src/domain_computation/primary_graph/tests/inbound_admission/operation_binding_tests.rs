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
    let receipt = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_other_dispatch(&phase, 117, "second-operation-original")
        })
        .expect("the fixture read owns its installed host call");
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

#[test]
fn workflow_settlement_joins_the_original_committed_operation_to_world_terminal() {
    let world = installed_world();
    let original = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 118, "workflow-original")
        })
        .expect("the fixture read owns its installed host call");
    let other = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 119, "workflow-other")
        })
        .expect("the fixture read owns its installed host call");
    let record = original
        .dispatch_outbox()
        .expect("original dispatch outbox");
    assert!(!world
        .application
        .resolve_guarded_workflow_external_settlement(&original)
        .expect("terminal index is available before completion"));

    let envelope = signed_envelope(record, [0xB2; 32], record.payload(), false);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(
                &world.verifier,
                &envelope,
                &super::super::fixture::live_scope()
            )
            .expect("installed source completes the original effect")
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let terminal = world
        .application
        .primary_provider
        .lookup_completed_inbound(record.correlation())
        .expect("canonical terminal index remains available")
        .expect("World performed a terminal completion");
    assert!(terminal.matches_original_dispatch(
        record,
        original.commit_reference(),
        original.committed_product_publication().composite_commit(),
        original
            .committed_product_publication()
            .product_incarnation(),
    ));
    assert!(!terminal.matches_original_dispatch(
        other.dispatch_outbox().unwrap(),
        other.commit_reference(),
        other.committed_product_publication().composite_commit(),
        other.committed_product_publication().product_incarnation(),
    ));
    assert!(world
        .application
        .resolve_guarded_workflow_external_settlement(&original)
        .expect("the original receipt resolves exact owner completion"));
    assert!(!world
        .application
        .resolve_guarded_workflow_external_settlement(&other)
        .expect("the other operation remains pending"));
}

#[test]
fn duplicate_callback_keeps_the_original_terminal_out_of_a_sibling_fork() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let original = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 120, "same-meaning-on-two-branches")
        })
        .expect("the fixture read owns its installed host call");
    let fork = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .branches()
                .fork(world.application.current_world())
                .components(|components| components.fork_relational().fork_signal())
                .create(&phase)
                .expect("the sibling product branch publishes")
        })
        .expect("the fixture read owns its installed host call");
    let sibling = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch_on(&phase, fork, 121, "same-meaning-on-two-branches")
        })
        .expect("the fixture read owns its installed host call");
    let original_record = original.dispatch_outbox().unwrap();
    let sibling_record = sibling.dispatch_outbox().unwrap();
    assert_eq!(original_record.payload(), sibling_record.payload());
    assert_ne!(original_record.correlation(), sibling_record.correlation());
    let envelope = signed_envelope(
        original_record,
        [0xB3; 32],
        original_record.payload(),
        false,
    );
    let request = super::super::fixture::live_scope();
    let first = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .expect("the original installed callback completes");
    assert_eq!(first.posture(), WorthQueryInboundReceiptPosture::Performed);
    let commits_after_first = super::tests::owner_commits(request_execution, &world);
    let duplicate = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .expect("the exact callback replays its World terminal");
    assert_eq!(
        duplicate.posture(),
        WorthQueryInboundReceiptPosture::Performed
    );
    assert_eq!(
        super::tests::owner_commits(request_execution, &world),
        commits_after_first
    );

    assert!(world
        .application
        .resolve_guarded_workflow_external_settlement(&original)
        .expect("the original owner still observes completion"));
    assert!(!world
        .application
        .resolve_guarded_workflow_external_settlement(&sibling)
        .expect("the sibling owner remains pending"));
    let terminal = world
        .application
        .primary_provider
        .lookup_completed_inbound(original_record.correlation())
        .unwrap()
        .unwrap();
    assert!(!terminal.matches_original_dispatch(
        sibling_record,
        sibling.commit_reference(),
        sibling.committed_product_publication().composite_commit(),
        sibling
            .committed_product_publication()
            .product_incarnation(),
    ));
}
