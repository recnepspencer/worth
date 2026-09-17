//! Production external-effect dispatch against a real, separate-process rail.
//!
//! Nothing here simulates the boundary. `notify_estate_death` runs the real
//! bank progression, Query co-commits the dispatch outbox inside the mutation
//! transaction, and the installed transport carries the correlation out over
//! TCP to a rail process spawned from `CARGO_BIN_EXE_bank-external-rail`.
//! Every posture asserted below is the one Query derived from what that
//! process actually did.

#[path = "external_effect_dispatch/committed_outbox.rs"]
mod committed_outbox;
#[path = "external_effect_dispatch/preparation_denial.rs"]
mod preparation_denial;
#[path = "external_effect_dispatch/publication_assertions.rs"]
pub(super) mod publication_assertions;
#[path = "external_effect_dispatch/rail_transport.rs"]
pub(crate) mod rail_transport;

use std::sync::Arc;
use std::time::Duration;

use bank_domain::{
    estate::{DeathNoticeStatus, ESTATE_DEATH_NOTICE_RAIL},
    proposals::BankIdempotencyKey,
};
use bank_external_rail::test_control::FaultScript;
use bank_external_rail::{
    LedgerStatus, RailCorrelation, RailProcessHandle, RailProtocolSupportProfile,
};
use bank_server::{queries, BankCommitReceipt, BankMutationCommitOutcome, BankReadControls};
use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedExternalEffectFailure, WorthQueryPublishedExternalEffectPostureKind,
    WorthQueryPublishedUnsupportedProtocolVersionPosture,
};

use self::rail_transport::{spawn_rail, spawn_rail_with_protocol_support, BankEstateRailTransport};
use super::notify_death::fixture::{notification_world, NotificationFixture};
use crate::support::request_scope;

const PATIENT: Duration = Duration::from_secs(5);
const IMPATIENT: Duration = Duration::from_millis(40);

struct DispatchWorld {
    fixture: NotificationFixture,
    transport: Arc<BankEstateRailTransport>,
    _rail: RailProcessHandle,
}

#[test]
fn a_succeeding_rail_completes_and_no_fault_ever_reports_completion() {
    let expectations: [(
        &str,
        FaultScript,
        Duration,
        Option<WorthQueryPublishedExternalEffectFailure>,
    ); 6] = [
        ("succeed", FaultScript::Succeed, PATIENT, None),
        (
            "acknowledge-only",
            FaultScript::AcknowledgeWithoutCompleting,
            PATIENT,
            None,
        ),
        (
            "duplicate-ack",
            FaultScript::DuplicateAcknowledgement,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement),
        ),
        (
            "slow-completion",
            FaultScript::CompleteAfterDelay { delay_millis: 400 },
            IMPATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::Timeout),
        ),
        (
            "lost-response",
            FaultScript::CommitThenLoseResponse,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::LostResponse),
        ),
        (
            "disappearance",
            FaultScript::DisappearMidDispatch,
            PATIENT,
            Some(WorthQueryPublishedExternalEffectFailure::Disconnect),
        ),
    ];

    for (scenario, script, frame_timeout, expected_fault) in expectations {
        assert_dispatch_scenario(scenario, script, frame_timeout, expected_fault);
    }
}

#[test]
fn unsupported_version_survives_the_rail_adapter_and_publication_boundary() {
    let world =
        dispatch_world_with_protocol_support("predating-v1", RailProtocolSupportProfile::V2Only);
    let receipt = world.commit_notification(52);

    assert_eq!(
        receipt
            .external_dispatch_posture()
            .and_then(|posture| posture.failure()),
        Some(
            WorthQueryPublishedExternalEffectFailure::UnsupportedProtocolVersion {
                produced: 1,
                posture: WorthQueryPublishedUnsupportedProtocolVersionPosture::PredatesWindow,
            }
        )
    );
    assert_eq!(world.transport.admission_count(), 0);
    assert_eq!(
        world.transport.ledger_status(&rail_correlation(&world)),
        LedgerStatus::NoRecord
    );
}

fn assert_dispatch_scenario(
    scenario: &str,
    script: FaultScript,
    frame_timeout: Duration,
    expected_fault: Option<WorthQueryPublishedExternalEffectFailure>,
) {
    let world = dispatch_world(scenario);
    world.transport.under(script, frame_timeout);
    let receipt = world.commit_notification(41);
    assert!(receipt.co_committed_dispatch_outbox());
    assert_eq!(world.transport.attempts().len(), 1);
    publication_assertions::assert_dispatch_publication_and_work(&receipt, scenario);

    let posture = receipt
        .external_dispatch_posture()
        .expect("a co-committed effect must be dispatched");
    match script {
        FaultScript::Succeed => assert_eq!(
            posture.kind(),
            WorthQueryPublishedExternalEffectPostureKind::Completed
        ),
        FaultScript::AcknowledgeWithoutCompleting => assert_eq!(
            posture.kind(),
            WorthQueryPublishedExternalEffectPostureKind::Acknowledged
        ),
        _ => assert_eq!(posture.failure(), expected_fault),
    }
    if expected_fault.is_some() {
        assert_ne!(
            posture.kind(),
            WorthQueryPublishedExternalEffectPostureKind::Completed,
            "{scenario}: a fault must never produce external completion"
        );
    }
}

/// Q8.25-C3: the rail learns what the notice *means*, not merely that one
/// happened.
///
/// The assertion is against the rail's own ledger, read back over a fresh TCP
/// connection to a different OS process, which holds a notice its own decoder
/// produced from the bytes the outbox co-committed. Before slice 9B the wire
/// carried a correlation family and a 32-byte token and nothing else: the
/// estate, the notice, and the subject stopped at the outbox row, and any real
/// external owner would have had to look them up elsewhere or assume them.
/// Nothing in this file could have detected that, because nothing asked the
/// rail what it understood.
#[test]
fn the_rail_decodes_the_committed_notice_rather_than_echoing_a_correlation() {
    let world = dispatch_world("decoded-notice");
    world.transport.under(FaultScript::Succeed, PATIENT);
    let receipt = world.commit_notification(47);

    assert_eq!(
        receipt
            .external_dispatch_posture()
            .map(|posture| posture.kind()),
        Some(WorthQueryPublishedExternalEffectPostureKind::Completed),
        "a well-formed notice on a healthy rail completes"
    );

    let decoded = world
        .transport
        .ledger_notice(&rail_correlation(&world))
        .expect("the rail admits an attempt only by first decoding its payload");
    assert_eq!(
        decoded.estate(),
        world.fixture.estate.get(),
        "the rail read the estate this commit actually notified"
    );
    assert_eq!(
        decoded.notice(),
        world.fixture.notice.get(),
        "the rail read the exact death notice, not some other notice on the estate"
    );
    assert_eq!(
        decoded.subject(),
        world.fixture.deceased.get(),
        "the rail read the subject the notice is about"
    );
}

/// The rail holds no meaning it did not decode.
///
/// A correlation the rail never received cannot yield a notice, so a passing
/// `ledger_notice` assertion above cannot be satisfied by a rail that answers
/// optimistically.
#[test]
fn an_unknown_correlation_yields_no_notice_from_the_rail() {
    let world = dispatch_world("unknown-correlation");
    world.transport.under(FaultScript::Succeed, PATIENT);
    let _ = world.commit_notification(49);

    let stranger = RailCorrelation::new(ESTATE_DEATH_NOTICE_RAIL, vec![0xAB; 32]);
    assert_eq!(
        world.transport.ledger_notice(&stranger),
        None,
        "the rail reports meaning only for attempts it actually decoded"
    );
}

#[test]
fn late_reconciliation_observes_rail_completion_without_dispatching_again() {
    let world = dispatch_world("late-reconciliation");
    world.transport.under(
        FaultScript::CompleteAfterDelay { delay_millis: 300 },
        IMPATIENT,
    );
    let receipt = world.commit_notification(43);

    let posture = receipt
        .external_dispatch_posture()
        .expect("the declared effect must be dispatched");
    assert_eq!(
        posture.failure(),
        Some(WorthQueryPublishedExternalEffectFailure::Timeout)
    );

    std::thread::sleep(Duration::from_millis(700));
    let correlation = rail_correlation(&world);
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed,
        "the rail completes on its own schedule regardless of the caller's deadline"
    );
    assert_eq!(
        world.transport.attempts().len(),
        1,
        "reconciliation reads the rail's ledger; it never dispatches a second effect"
    );
    assert_eq!(
        world.notice_status(),
        DeathNoticeStatus::NotificationRequested,
        "the durable bank truth advanced exactly once"
    );
}

#[test]
fn a_duplicate_acknowledgement_never_advances_the_posture_twice() {
    let world = dispatch_world("duplicate-acknowledgement");
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, PATIENT);
    let binding = idempotency(45);
    let receipt = world.commit_with(binding.clone());
    let posture = receipt
        .external_dispatch_posture()
        .expect("the declared effect must be dispatched");
    assert_eq!(
        posture.failure(),
        Some(WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement)
    );

    let retry = world.notify(binding);
    let BankMutationCommitOutcome::AlreadyCommitted(recovered) = retry else {
        panic!("an equivalent retry must recover the commit: {retry:?}");
    };
    publication_assertions::assert_recovered_commit_axes(&receipt, &recovered);
    assert!(
        recovered.external_dispatch_posture().is_none(),
        "a recovered commit re-runs no dispatch and invents no posture"
    );
    assert_eq!(
        world.transport.attempts().len(),
        1,
        "the second acknowledgement must not become a second dispatch"
    );
    assert_eq!(
        world.transport.ledger_status(&rail_correlation(&world)),
        LedgerStatus::Acknowledged,
        "the rail never completed this attempt"
    );
}

impl DispatchWorld {
    fn commit_notification(&self, identity: u8) -> BankCommitReceipt {
        self.commit_with(idempotency(identity))
    }

    fn commit_with(&self, binding: BankIdempotencyKey) -> BankCommitReceipt {
        let outcome = self.notify(binding);
        let BankMutationCommitOutcome::Committed(receipt) = outcome else {
            panic!("the exact reported notice must commit: {outcome:?}");
        };
        receipt
    }

    fn notify(&self, binding: BankIdempotencyKey) -> BankMutationCommitOutcome {
        self.fixture
            .world
            .runtime
            .notify_estate_death_with_key(
                &self.fixture.authenticate_specialist(),
                self.fixture
                    .action(self.fixture.notice, self.fixture.deceased),
                &binding,
                &request_scope(),
            )
            .expect("the lawful death notification should reach commit")
    }

    fn notice_status(&self) -> DeathNoticeStatus {
        self.fixture
            .world
            .runtime
            .query(queries::estate_case(self.fixture.estate))
            .as_principal(&self.fixture.authenticate_specialist())
            .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
            .execute()
            .expect("the assigned specialist should observe the death notice")
            .rows()[0]
            .death_notice()
            .status()
    }
}

fn dispatch_world(scenario: &str) -> DispatchWorld {
    let rail = spawn_rail();
    install_dispatch_world(scenario, rail)
}

fn dispatch_world_with_protocol_support(
    scenario: &str,
    protocol_support: RailProtocolSupportProfile,
) -> DispatchWorld {
    let rail = spawn_rail_with_protocol_support(protocol_support);
    install_dispatch_world(scenario, rail)
}

fn install_dispatch_world(scenario: &str, rail: RailProcessHandle) -> DispatchWorld {
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let fixture = notification_world(
        &format!("external-effect-{scenario}"),
        DeathNoticeStatus::Reported,
    );
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("the bank installs its rail once per runtime");
    DispatchWorld {
        fixture,
        transport,
        _rail: rail,
    }
}

fn rail_correlation(world: &DispatchWorld) -> RailCorrelation {
    world
        .transport
        .attempts()
        .into_iter()
        .next()
        .expect("a dispatched effect reaches the rail with one correlation")
}

fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("external-notification-{identity}")).unwrap()
}
