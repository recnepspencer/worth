use std::num::NonZeroUsize;
use std::time::Duration;

use super::fixture::installed_world;
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundReceiptPosture,
};

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
        Err(WorthQueryInboundAdmissionDenial::MessageIdentityConflict),
    ));

    let altered_effect = signed_envelope(first_record, [0xa4; 32], b"altered", false);
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &altered_effect, &request,),
        Err(WorthQueryInboundAdmissionDenial::CorrelationAlreadyOwned),
    ));
    assert_eq!(super::tests::owner_commits(&world), commits_after_turnover);
}
