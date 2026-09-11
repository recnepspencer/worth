//! Courtroom D: committed outbox owner read and affinity.

use bank_domain::estate::ESTATE_DEATH_NOTICE_RAIL;
use bank_external_rail::test_control::FaultScript;
use bank_server::BankCommittedDispatchOutboxReadDenial;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

use super::{dispatch_world, PATIENT};

#[test]
fn committed_outbox_is_fresh_owner_truth_with_exact_commit_affinity() {
    let world = dispatch_world("committed-owner-read");
    world.transport.under(FaultScript::Succeed, PATIENT);
    let receipt = world.commit_notification(48);
    let observed = world
        .fixture
        .world
        .runtime
        .observe_committed_dispatch_outbox(&receipt)
        .expect("the Query provider can read its current Relational owner view")
        .expect("the declared effect committed one outbox row");

    let mut expected_payload = Vec::with_capacity(24);
    expected_payload.extend_from_slice(&world.fixture.estate.get().to_be_bytes());
    expected_payload.extend_from_slice(&world.fixture.notice.get().to_be_bytes());
    expected_payload.extend_from_slice(&world.fixture.deceased.get().to_be_bytes());
    assert_eq!(observed.correlation_family(), ESTATE_DEATH_NOTICE_RAIL);
    assert_eq!(observed.effect(), "EstateDeathNotificationEffect");
    assert_eq!(
        observed.protocol_identity(),
        &BoundaryProtocolIdentity::new("bank.estate.death-notification")
    );
    assert_eq!(observed.protocol_version(), BoundaryProtocolVersion::new(1));
    assert_eq!(observed.maximum_payload_bytes(), 24);
    assert_eq!(observed.payload(), expected_payload);
    assert_eq!(
        observed.correlation(),
        world.transport.attempts()[0].token(),
        "the committed owner row supplied the exact rail correlation"
    );
}

#[test]
fn another_runtime_without_the_exact_commit_cannot_observe_a_receipts_committed_outbox() {
    let owner = dispatch_world("owner-runtime");
    owner.transport.under(FaultScript::Succeed, PATIENT);
    let receipt = owner.commit_notification(50);
    let stranger = dispatch_world("stranger-runtime");

    assert_eq!(
        stranger
            .fixture
            .world
            .runtime
            .observe_committed_dispatch_outbox(&receipt),
        Err(BankCommittedDispatchOutboxReadDenial::ExactCommitUnavailable)
    );
}
