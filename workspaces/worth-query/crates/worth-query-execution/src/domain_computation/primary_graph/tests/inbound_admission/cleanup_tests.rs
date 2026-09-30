use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use super::fixture::installed_world;
use super::schema::WideNotifyOperation;
use super::verifier::TestVerifier;
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::application_aftermath::WorthQueryInboundPublicationClaim;
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundReceiptPosture,
};

#[test]
fn one_cleanup_page_reclaims_one_of_two_terminals_but_never_pending_custody() {
    let world = installed_world();
    let request = super::super::fixture::live_scope();
    let wide_operation = world
        .application
        .installed_schema()
        .installed_operation(WideNotifyOperation::reference())
        .unwrap();
    let wide_handle = world
        .application
        .install_inbound_occurrence_verifier(&wide_operation, Arc::new(TestVerifier))
        .unwrap();

    for (seed, message) in [(51_u64, [0xc1; 32]), (52, [0xc2; 32])] {
        let completed = world.commit_wide_dispatch(seed, "settled-short");
        let record = completed.dispatch_outbox().unwrap();
        let envelope = signed_envelope_for_seconds(record, message, record.payload(), false, 1);
        assert_eq!(
            world
                .application
                .receive_inbound_occurrence(&wide_handle, &envelope, &request)
                .unwrap()
                .posture(),
            WorthQueryInboundReceiptPosture::Performed,
        );
    }
    let pending = world.commit_wide_dispatch(53, "pending-same-operation");
    let pending_record = pending.dispatch_outbox().unwrap();
    let pending_envelope =
        signed_envelope(pending_record, [0xc3; 32], pending_record.payload(), false);
    assert!(world
        .application
        .admit_inbound_occurrence(&wide_handle, &pending_envelope)
        .is_ok());
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&wide_handle)
            .unwrap()
            .accepted_occurrences(),
        3
    );

    std::thread::sleep(Duration::from_secs(2));
    let page = NonZeroUsize::new(1).unwrap();
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&wide_handle, page)
            .unwrap()
            .reclaimed(),
        1
    );
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&wide_handle)
            .unwrap()
            .accepted_occurrences(),
        2,
        "one-work page leaves the second eligible terminal and pending slot charged",
    );
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&wide_handle, page)
            .unwrap()
            .reclaimed(),
        1
    );
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&wide_handle, page)
            .unwrap()
            .reclaimed(),
        0
    );
    assert_eq!(
        world
            .application
            .observe_inbound_cost(&wide_handle)
            .unwrap()
            .accepted_occurrences(),
        1
    );
}

#[test]
fn expired_settled_terminal_reclaims_one_finite_accepted_slot() {
    let world = installed_world();
    let first = world.commit_dispatch(41, "notice-short-validity");
    let first_record = first.dispatch_outbox().unwrap();
    let first_envelope =
        signed_envelope_for_seconds(first_record, [0xa1; 32], first_record.payload(), false, 1);
    let request = super::super::fixture::live_scope();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &first_envelope, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
    );
    let accepted_before_cleanup = world
        .application
        .retained_accepted_for_cleanup_test(first_record.correlation())
        .expect("performed owner retains accepted custody before expiry");
    let one = NonZeroUsize::new(1).unwrap();
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&world.verifier, one)
            .unwrap()
            .reclaimed(),
        0,
        "signed replay is still valid",
    );

    std::thread::sleep(Duration::from_secs(2));
    assert_eq!(
        world
            .application
            .cleanup_completed_inbound_occurrences(&world.verifier, one)
            .unwrap()
            .reclaimed(),
        1,
    );
    let (claim, retryable) = world
        .application
        .stale_publication_claim_for_cleanup_test(&accepted_before_cleanup);
    assert_eq!(
        claim,
        WorthQueryInboundPublicationClaim::Gone,
        "a duplicate holding an Arc across cleanup gets a typed gone result",
    );
    assert!(!retryable);
    let second = world.commit_dispatch(42, "notice-after-turnover");
    let second_record = second.dispatch_outbox().unwrap();
    let second_envelope =
        signed_envelope(second_record, [0xa2; 32], second_record.payload(), false);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(
                &world.verifier,
                &second_envelope,
                &super::super::fixture::live_scope(),
            )
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::Performed,
        "the finite accepted slot must turn over after lawful cleanup",
    );

    let fresh_identity = signed_envelope(first_record, [0xa3; 32], first_record.payload(), false);
    let commits_after_turnover = super::tests::owner_commits(&world);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &fresh_identity, &request)
            .unwrap()
            .posture(),
        WorthQueryInboundReceiptPosture::AlreadyCompleted,
        "a new signed message for the completed effect must find terminal history",
    );
    assert_eq!(super::tests::owner_commits(&world), commits_after_turnover);

    let reused_message_identity =
        signed_envelope(first_record, [0xa1; 32], first_record.payload(), false);
    assert!(matches!(
        world.application.receive_inbound_occurrence(
            &world.verifier,
            &reused_message_identity,
            &request,
        ),
        Err(WorthQueryInboundAdmissionDenial::AuthenticatedPermanent(proof))
            if proof.kind() == crate::domain_computation::primary_graph::WorthQueryInboundPermanentDenialKind::MessageIdentityConflict
                && proof.message_identity() == &[0xa1; 32]
                && proof.envelope_digest() == &<[u8; 32]>::from(Sha256::digest(&reused_message_identity)),
    ));

    let altered_effect = signed_envelope(first_record, [0xa4; 32], b"altered", false);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &altered_effect, &request,),
        Err(WorthQueryInboundAdmissionDenial::AuthenticatedPermanent(proof))
            if proof.kind() == crate::domain_computation::primary_graph::WorthQueryInboundPermanentDenialKind::CorrelationAlreadyOwned,
    ));
    assert_eq!(super::tests::owner_commits(&world), commits_after_turnover);
}
