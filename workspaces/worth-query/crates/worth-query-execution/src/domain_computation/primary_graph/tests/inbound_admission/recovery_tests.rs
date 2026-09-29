use super::fixture::installed_world;
use super::tests::{completion_records, owner_commits, product_commit};
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundReceiptPosture as Posture,
};

#[test]
fn retained_world_unpublished_completion_settles_and_publishes_without_recreating_effect() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(41, "notice-recovery");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xa1; 32], record.payload(), false);
    let before_relational = owner_commits(&world);
    let before_product = product_commit(&world);
    let request = super::super::fixture::live_scope();
    world.application.fail_next_durable_append_for_test();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::AcceptedPending,
    );
    assert_eq!(owner_commits(&world), before_relational + 1);
    assert_eq!(product_commit(&world), before_product);
    assert_eq!(completion_records(&world), 0);

    assert_eq!(
        world
            .application
            .progress_retained_inbound_occurrence(*record.correlation().bytes(), &request,)
            .unwrap(),
        Posture::Performed,
    );
    assert_eq!(owner_commits(&world), before_relational + 1);
    assert_ne!(product_commit(&world), before_product);
    assert_eq!(completion_records(&world), 1);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::Performed,
    );
}

#[test]
fn revoked_source_preserves_accepted_world_recovery_without_new_consumption() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(42, "notice-revoked-recovery");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xa2; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    world.application.fail_next_durable_append_for_test();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::AcceptedPending,
    );
    let relational = owner_commits(&world);
    let product = product_commit(&world);
    world
        .application
        .revoke_inbound_occurrence_source(&world.verifier)
        .unwrap();
    assert!(matches!(
        world
            .application
            .progress_retained_inbound_occurrence(*record.correlation().bytes(), &request),
        Err(Denial::SourceRevoked),
    ));
    assert!(matches!(
        world
            .application
            .progress_retained_inbound_occurrence(*record.correlation().bytes(), &request),
        Err(Denial::SourceRevoked),
    ));
    assert_eq!(owner_commits(&world), relational);
    assert_eq!(product_commit(&world), product);
    assert_eq!(completion_records(&world), 0);
}

#[test]
fn owner_maintenance_finishes_accepted_recovery_after_signed_envelope_expires() {
    let world = installed_world();
    let dispatch = world.commit_dispatch(43, "notice-expired-recovery");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope_for_seconds(record, [0xa3; 32], record.payload(), false, 1);
    let request = super::super::fixture::live_scope();
    world.application.fail_next_durable_append_for_test();
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::AcceptedPending,
    );
    std::thread::sleep(std::time::Duration::from_secs(2));
    assert!(matches!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request),
        Err(Denial::Verification(_)) | Err(Denial::Expired),
    ));
    let report = world
        .application
        .maintain_inbound_occurrences(
            &world.verifier,
            std::num::NonZeroUsize::new(1).unwrap(),
            &request,
        )
        .unwrap();
    assert_eq!(report.performed(), 1);
    assert_eq!(report.blocked(), 0);
    assert_eq!(completion_records(&world), 1);
}
