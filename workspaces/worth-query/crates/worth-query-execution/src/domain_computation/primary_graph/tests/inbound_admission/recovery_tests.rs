use super::fixture::installed_world;
use super::tests::{completion_records, owner_commits, product_commit};
use super::verifier::{signed_envelope, signed_envelope_for_seconds};
use crate::domain_computation::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundPendingReason as Reason,
    WorthQueryInboundReceiptPosture as Posture,
};

#[test]
fn retained_world_unpublished_completion_settles_and_publishes_without_recreating_effect() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 41, "notice-recovery")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xa1; 32], record.payload(), false);
    let before_relational = owner_commits(execution, &world);
    let before_product = product_commit(&world);
    let request = super::super::fixture::live_scope();
    world
        .application
        .fail_next_durable_append_for_test(execution);
    let receipt = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .unwrap();
    assert_eq!(receipt.posture(), Posture::AcceptedPending);
    assert_eq!(receipt.pending_reason(), Some(Reason::RetainedUnpublished));
    assert_eq!(owner_commits(execution, &world), before_relational + 1);
    assert_eq!(product_commit(&world), before_product);
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
                        &request,
                    )
                    .unwrap(),
                Posture::Performed,
            );
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(owner_commits(execution, &world), before_relational + 1);
    assert_ne!(product_commit(&world), before_product);
    assert_eq!(completion_records(execution, &world), 1);
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
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 42, "notice-revoked-recovery")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope(record, [0xa2; 32], record.payload(), false);
    let request = super::super::fixture::live_scope();
    world
        .application
        .fail_next_durable_append_for_test(request_execution);
    assert_eq!(
        world
            .application
            .receive_inbound_occurrence(&world.verifier, &envelope, &request)
            .unwrap()
            .posture(),
        Posture::AcceptedPending,
    );
    let relational = owner_commits(request_execution, &world);
    let product = product_commit(&world);
    world
        .application
        .revoke_inbound_occurrence_source(&world.verifier)
        .unwrap();
    let retained = world
        .application
        .receive_inbound_occurrence(&world.verifier, &envelope, &request)
        .expect("authenticated accepted custody remains acknowledgeable after revocation");
    assert_eq!(retained.posture(), Posture::AcceptedPending);
    assert_eq!(retained.pending_reason(), Some(Reason::SourceRevoked));
    assert!(retained.requires_maintenance_cue());
    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world.application.progress_retained_inbound_occurrence(
                    &phase.borrowed_request_for_test(),
                    phase,
                    *record.correlation().bytes(),
                    &request
                ),
                Err(Denial::SourceRevoked),
            ));
        })
        .expect("fixture owner admits its advancement");

    world
        .application
        .with_host_advancement(|active_phase| {
            let phase = &active_phase;
            assert!(matches!(
                world.application.progress_retained_inbound_occurrence(
                    &phase.borrowed_request_for_test(),
                    phase,
                    *record.correlation().bytes(),
                    &request
                ),
                Err(Denial::SourceRevoked),
            ));
        })
        .expect("fixture owner admits its advancement");

    assert_eq!(owner_commits(request_execution, &world), relational);
    assert_eq!(product_commit(&world), product);
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
}

#[test]
fn owner_maintenance_finishes_accepted_recovery_after_signed_envelope_expires() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_world();
    let dispatch = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.commit_dispatch(&phase, 43, "notice-expired-recovery")
        })
        .expect("the fixture read owns its installed host call");
    let record = dispatch.dispatch_outbox().unwrap();
    let envelope = signed_envelope_for_seconds(record, [0xa3; 32], record.payload(), false, 1);
    let request = super::super::fixture::live_scope();
    world
        .application
        .fail_next_durable_append_for_test(execution);
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
    assert_eq!(completion_records(execution, &world), 1);
}
