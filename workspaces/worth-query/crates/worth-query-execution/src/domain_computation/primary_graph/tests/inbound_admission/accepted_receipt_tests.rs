use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};

use super::fixture::installed_world;
use super::schema::WideNotifyOperation;
use super::tests::completion_records;
use super::verifier::{signed_envelope, signed_envelope_for_seconds, TestVerifier};
use crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryInboundAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryInboundPendingReason as Reason, WorthQueryInboundReceiptPosture as Posture,
};

#[test]
fn verified_second_callback_is_accepted_while_publication_slot_is_busy() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(WideNotifyOperation::reference())
        .unwrap();
    let handle = world
        .application
        .install_inbound_occurrence_verifier(&operation, Arc::new(TestVerifier))
        .unwrap();
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_wide_dispatch(&phase, 151, "first-held-publication")
        })
        .expect("the fixture read owns its installed host call");
    let first_record = first.dispatch_outbox().unwrap();
    let first_envelope = signed_envelope(first_record, [0xd1; 32], first_record.payload(), false);
    let WorthQueryInboundAdmission::New(first_accepted, true) = world
        .application
        .admit_inbound_occurrence(&handle, &first_envelope)
        .unwrap()
    else {
        panic!("first authenticated owner must reserve the publication slot");
    };
    let second = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_wide_dispatch(&phase, 152, "second-queued-publication")
        })
        .expect("the fixture read owns its installed host call");
    let second_record = second.dispatch_outbox().unwrap();
    let second_envelope =
        signed_envelope(second_record, [0xd2; 32], second_record.payload(), false);
    let request = super::super::fixture::live_scope();
    let receipt = world
        .application
        .receive_inbound_occurrence(&handle, &second_envelope, &request)
        .unwrap();
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(
        receipt.pending_reason(),
        Some(Reason::PublicationAtCapacity)
    );
    assert!(receipt.requires_maintenance_cue());
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                completion_records(&host_custody.borrowed_request_for_test(), &world),
                0
            )
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&handle)
            .unwrap()
            .accepted_occurrences(),
        2
    );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;

            assert_eq!(
                world
                    .application
                    .progress_accepted_inbound_occurrence(
                        &phase.borrowed_request_for_test(),
                        phase,
                        first_accepted,
                        &request
                    )
                    .unwrap(),
                Posture::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world
                    .application
                    .progress_retained_inbound_occurrence(
                        &phase.borrowed_request_for_test(),
                        phase,
                        *second_record.correlation().bytes(),
                        &request
                    )
                    .unwrap(),
                Posture::Performed
            );
        })
        .expect("fixture owner admits its advancement");

    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            assert_eq!(
                completion_records(&host_custody.borrowed_request_for_test(), &world),
                2
            )
        })
        .expect("the fixture read owns its installed host call");
}

#[test]
fn signed_validity_span_exceeding_installed_window_is_not_called_expired() {
    let world = installed_world();
    let dispatched = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 153, "excess-validity-span")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatched.dispatch_outbox().unwrap();
    let envelope = signed_envelope_for_seconds(record, [0xd3; 32], record.payload(), false, 61);
    let request = super::super::fixture::live_scope();
    assert!(matches!(
        world.application.receive_inbound_occurrence(&world.verifier, &envelope, &request),
        Err(crate::domain_computation::primary_graph::WorthQueryInboundAdmissionDenial::ValidityWindowExceeded)
    ));
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap()
            .accepted_occurrences(),
        0
    );
}

#[test]
fn retained_public_correlation_cannot_accept_after_signed_cutoff() {
    let world = installed_world();
    let dispatched = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 155, "phase-cutoff")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatched.dispatch_outbox().unwrap();
    let envelope = signed_envelope_for_seconds(record, [0xd5; 32], record.payload(), false, 1);
    let correlated = world
        .application
        .authenticate_inbound_occurrence(&world.verifier, &envelope)
        .unwrap()
        .correlate()
        .unwrap();
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(matches!(
        correlated.accept(),
        Err(crate::domain_computation::primary_graph::WorthQueryInboundAdmissionDenial::Expired)
    ));
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&world.verifier)
            .unwrap()
            .accepted_occurrences(),
        0
    );
}

#[test]
fn dropping_admitted_public_phase_releases_publication_slot_but_keeps_custody() {
    let world = installed_world();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(WideNotifyOperation::reference())
        .unwrap();
    let handle = world
        .application
        .install_inbound_occurrence_verifier(&operation, Arc::new(TestVerifier))
        .unwrap();
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_wide_dispatch(&phase, 156, "abandoned-public-phase")
        })
        .expect("the fixture read owns its installed host call");
    let record = first.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xd6; 32], record.payload(), false);
    let admitted = world
        .application
        .authenticate_inbound_occurrence(&handle, &envelope)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();
    drop(admitted);
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&handle)
            .unwrap()
            .accepted_occurrences(),
        1
    );
    let second = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_wide_dispatch(&phase, 157, "next-publication")
        })
        .expect("the fixture read owns its installed host call");
    let second_record = second.dispatch_outbox().unwrap();
    let second_envelope =
        signed_envelope(second_record, [0xd7; 32], second_record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&handle, &second_envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world
                    .application
                    .progress_retained_inbound_occurrence(
                        &phase.borrowed_request_for_test(),
                        phase,
                        *record.correlation().bytes(),
                        &request
                    )
                    .unwrap(),
                Posture::Performed
            );
        })
        .expect("fixture owner admits its advancement");
}

#[test]
fn old_executed_phase_drop_cannot_release_a_new_publication_claim() {
    let world = installed_world();
    let dispatched = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 158, "reclaimed-publication-claim")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatched.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xd8; 32], record.payload(), false);
    let mut old_phase = world
        .application
        .authenticate_inbound_occurrence(&world.verifier, &envelope)
        .unwrap()
        .correlate()
        .unwrap()
        .accept()
        .unwrap();

    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let cancelled = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(60),
        cancellation.token(),
    );
    let receipt = old_phase.progress_for_test(&cancelled).unwrap();
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(receipt.pending_reason(), Some(Reason::OwnerRetryRequired));

    let reclaimed = world
        .application
        .admit_inbound_occurrence(&world.verifier, &envelope)
        .unwrap();
    let WorthQueryInboundAdmission::Duplicate(accepted, WorthQueryInboundPublicationClaim::Claimed) =
        reclaimed
    else {
        panic!("retryable custody must grant the later publication claim");
    };
    drop(old_phase);
    assert!(matches!(
        world
            .application
            .admit_inbound_occurrence(&world.verifier, &envelope)
            .unwrap(),
        WorthQueryInboundAdmission::Duplicate(_, WorthQueryInboundPublicationClaim::Publishing)
    ));
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert_eq!(
                world
                    .application
                    .progress_accepted_inbound_occurrence(
                        &phase.borrowed_request_for_test(),
                        phase,
                        accepted,
                        &super::super::fixture::live_scope()
                    )
                    .unwrap(),
                Posture::Performed
            );
        })
        .expect("fixture owner admits its advancement");
}
