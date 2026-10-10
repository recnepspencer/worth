use sha2::{Digest, Sha256};

use super::fixture::installed_world;
use super::verifier::{signed_envelope, signed_envelope_for_second_key_epoch};
use crate::domain_computation::application_aftermath::WorthQueryInboundVerificationDenial;
use crate::domain_computation::primary_graph::application_runtime::WorthQueryExternalDispatchAdmissionDenial;
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundReceiptPosture as Posture,
};

mod helpers;
pub(super) use helpers::{completion_records, owner_commits, product_commit};

#[test]
fn same_message_identity_under_a_new_authenticated_key_epoch_is_already_completed() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 12, "epoch-rotated-completion")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let first = signed_envelope(record, [0x92; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &first, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let before = owner_commits(request_execution, &world);
    let rotated = signed_envelope_for_second_key_epoch(record, [0x92; 32]);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &rotated, &request)
            .unwrap()
            .posture(),
        Posture::AlreadyCompleted
    );
    assert_eq!(owner_commits(request_execution, &world), before);
}

#[test]
fn installed_verifier_completes_a_genuine_dispatch_once_and_replays_its_terminal_result() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 11, "notice-one")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0x91; 32], record.payload(), false);
    let before_relational = owner_commits(execution, &world);
    let before_product = product_commit(&world);
    assert_eq!(completion_records(execution, &world), 0);
    let request = super::super::fixture::live_scope();
    let first = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert_eq!(first.posture(), Posture::Performed);
    assert_eq!(first.message_identity(), &[0x91; 32]);
    let envelope_digest: [u8; 32] = Sha256::digest(&envelope).into();
    assert_eq!(first.envelope_digest(), &envelope_digest);
    assert_eq!(owner_commits(execution, &world), before_relational + 1);
    assert_ne!(product_commit(&world), before_product);
    assert_eq!(completion_records(execution, &world), 1);
    let completed_product = product_commit(&world);

    let replay = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert_eq!(replay.posture(), Posture::Performed);
    assert_eq!(replay.message_identity(), first.message_identity());
    assert_eq!(owner_commits(execution, &world), before_relational + 1);
    assert_eq!(product_commit(&world), completed_product);
    assert_eq!(completion_records(execution, &world), 1);
}

#[test]
fn performed_inbound_completion_closes_later_dispatch_admission() {
    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 21, "notice-terminal")
        })
        .expect("the fixture read owns its installed host call");
    let committed = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .application
                .observe_committed_dispatch_outbox(&phase, &dispatch)
                .unwrap()
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let envelope = signed_envelope(
        dispatch.dispatch_outbox().unwrap(),
        [0x95; 32],
        dispatch.dispatch_outbox().unwrap().payload(),
        false,
    );
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    assert!(matches!(
        world.application.admit_external_dispatch_attempt(committed),
        Err(WorthQueryExternalDispatchAdmissionDenial::AlreadyCompleted)
    ));
}

#[path = "tests/lost_terminal_index_denies_until_bounded_owner_reconstruction.rs"]
mod lost_terminal_index_denies_until_bounded_owner_reconstruction;

#[test]
fn cleanup_verification_of_a_completed_inbound_keeps_the_terminal_owner() {
    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 32, "notice-durable-cleanup")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let correlation = *record.correlation();
    let envelope = signed_envelope(record, [0x97; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let sealed = world
        .application
        .primary_provider
        .lookup_completed_inbound(&correlation)
        .unwrap()
        .expect("the terminal owner holds the completion");
    world
        .application
        .verify_completed_inbound_for_cleanup(&[correlation], u64::MAX)
        .expect("World-paired durable row permits exact expiry turnover");
    assert_eq!(
        world
            .application
            .primary_provider
            .lookup_completed_inbound(&correlation)
            .unwrap(),
        Some(sealed)
    );
    world
        .application
        .verify_completed_inbound_for_cleanup(&[correlation], u64::MAX)
        .expect("retry after custody turnover remains idempotent");
}

#[test]
fn authenticated_payload_and_domain_mismatch_are_denied_before_custody() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 12, "notice-two")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let before_relational = owner_commits(execution, &world);
    let before_product = product_commit(&world);
    let request = super::super::fixture::live_scope();

    let wrong_domain = signed_envelope(record, [0x92; 32], record.payload(), true);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &wrong_domain, &request),
        Err(Denial::IncompatibleMeaning)
    ));
    let wrong_payload = signed_envelope(record, [0x92; 32], b"other-notice", false);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &wrong_payload, &request),
        Err(Denial::UnsupportedOutbox)
    ));
    let mut tampered = signed_envelope(record, [0x92; 32], record.payload(), false);
    *tampered.last_mut().unwrap() ^= 1;
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &tampered, &request),
        Err(Denial::Verification(
            WorthQueryInboundVerificationDenial::AuthenticationFailed
        ))
    ));
    assert_eq!(owner_commits(execution, &world), before_relational);
    assert_eq!(product_commit(&world), before_product);
    assert_eq!(completion_records(execution, &world), 0);

    let valid = signed_envelope(record, [0x92; 32], record.payload(), false);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &valid, &request)
            .unwrap()
            .posture(),
        Posture::Performed,
        "rejected signed envelopes must not occupy the one accepted slot"
    );
}

#[test]
fn finite_accepted_custody_denies_a_second_lawful_completion_before_ack() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let first = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 13, "notice-three")
        })
        .expect("the fixture read owns its installed host call");
    let first_record = first.dispatch_outbox().unwrap();
    let first_envelope = signed_envelope(first_record, [0x93; 32], first_record.payload(), false);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &first_envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed
    );
    let second = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 14, "notice-four")
        })
        .expect("the fixture read owns its installed host call");
    let second_record = second.dispatch_outbox().unwrap();
    let second_envelope =
        signed_envelope(second_record, [0x94; 32], second_record.payload(), false);
    let before_relational = owner_commits(execution, &world);
    let before_product = product_commit(&world);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &second_envelope, &request),
        Err(Denial::CapacityExhausted)
    ));
    assert_eq!(owner_commits(execution, &world), before_relational);
    assert_eq!(product_commit(&world), before_product);
    assert_eq!(completion_records(execution, &world), 1);
}

#[test]
fn retired_and_revoked_sources_cannot_admit_new_completion() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let host_request = crate::test_request_policy::serial_request();
        let execution = &worth_execution::ExecutionRequest::serial(&host_request);

        for revoked in [false, true] {
            let world = installed_world();
            let dispatch = world.commit_dispatch(
                &host_custody.phase_for_application(&world.application),
                31,
                "notice-source-state",
            );
            let record = dispatch.dispatch_outbox().unwrap();
            let envelope = signed_envelope(record, [0x96; 32], record.payload(), false);
            if revoked {
                world
                    .application
                    .revoke_inbound_occurrence_source(&world.verifier)
                    .unwrap();
            } else {
                world
                    .application
                    .retire_inbound_occurrence_source(&world.verifier)
                    .unwrap();
            }
            let request = super::super::fixture::live_scope();
            let expected = if revoked {
                Denial::SourceRevoked
            } else {
                Denial::SourceRetired
            };
            assert!(matches!(
                world
                    .application
                    .receive_inbound_occurrence(&world.verifier, &envelope, &request),
                Err(actual) if actual == expected
            ));
            assert_eq!(completion_records(execution, &world), 0);
        }
    })
}
