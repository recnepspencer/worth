//! Production external-effect dispatch against a real, separate-process rail.
//!
//! Nothing here simulates the boundary. `notify_estate_death` runs the real
//! bank progression, Query co-commits the dispatch outbox inside the mutation
//! transaction, and the installed transport carries the correlation out over
//! TCP to a rail process spawned from `CARGO_BIN_EXE_bank-external-rail`.
//! Every posture asserted below is the one Query derived from what that
//! process actually did.

#[path = "external_effect_dispatch/committed_outbox.rs"]
pub mod committed_outbox;
#[path = "external_effect_dispatch/preparation_denial.rs"]
pub mod preparation_denial;
#[path = "external_effect_dispatch/publication_assertions.rs"]
pub mod publication_assertions;
#[path = "external_effect_dispatch/rail_transport.rs"]
pub mod rail_transport;

pub use std::sync::Arc;
pub use std::time::Duration;

pub use bank_domain::{
    estate::{DeathNoticeStatus, ESTATE_DEATH_NOTICE_RAIL},
    proposals::BankIdempotencyKey,
};
pub use bank_external_rail::test_control::FaultScript;
pub use bank_external_rail::{
    LedgerStatus, RailCorrelation, RailProcessHandle, RailProtocolSupportProfile,
};
pub use bank_server::{queries, BankCommitReceipt, BankReadControls};
pub use worth_query_host::facade::publication::application_aftermath::{
    WorthQueryPublishedExternalEffectFailure, WorthQueryPublishedExternalEffectPostureKind,
    WorthQueryPublishedUnsupportedProtocolVersionPosture,
};

pub use self::rail_transport::{
    spawn_rail, spawn_rail_with_protocol_support, BankEstateRailTransport,
};
pub use super::notify_death::fixture::{notification_world, NotificationFixture};
pub use crate::facade::support::request_scope;

pub const PATIENT: Duration = Duration::from_secs(5);
pub const IMPATIENT: Duration = Duration::from_millis(40);

pub struct DispatchWorld {
    pub fixture: NotificationFixture,
    pub transport: Arc<BankEstateRailTransport>,
    pub _rail: RailProcessHandle,
}

pub fn assert_dispatch_scenario(
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

impl DispatchWorld {
    pub fn commit_notification(&self, identity: u8) -> BankCommitReceipt {
        self.commit_with(idempotency(identity))
    }

    pub fn commit_with(&self, binding: BankIdempotencyKey) -> BankCommitReceipt {
        let (receipt, fresh) = self.notify(binding);
        assert!(fresh, "native fresh completion");
        receipt
    }

    pub fn notify(&self, binding: BankIdempotencyKey) -> (BankCommitReceipt, bool) {
        {
            let source_actor = &self.fixture.authenticate_specialist();
            let source_scope = &request_scope();
            let source_key = &binding;
            let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
                self.fixture
                    .action(self.fixture.notice, self.fixture.deceased),
            )
            .expect("typed estate command preserves the exact variant");
            let source_request = self
                .fixture
                .world
                .runtime
                .request(source_actor, source_scope);
            let source_operation = self
                .fixture
                .world
                .runtime
                .mutate(bank_server::mutations::notify_estate_death(source_intent))
                .as_principal(source_actor)
                .controls(bank_server::BankMutationControls::new(
                    (*source_scope).clone(),
                    (*source_key).clone(),
                ))
                .retain_operation()
                .unwrap_or_else(|cause| panic!("exact declared estate construction: {cause:?}"));
            let worth_query_host::facade::application_entry::WorthQuerySourceOnlyAdvance::Completed(
                source_completed,
            ) = source_operation.advance(&source_request)
            else {
                panic!("native source completion required")
            };
            let fresh = matches!(
                source_completed.evidence(),
                worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
            );
            let closed_source = source_completed
                .close_with_evidence(&source_request)
                .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
            (
                bank_server::BankCommitReceipt::from_closed_source(closed_source),
                fresh,
            )
        }
    }

    pub fn notice_status(&self) -> DeathNoticeStatus {
        self.fixture
            .world
            .runtime
            .query(queries::estate_case(self.fixture.estate))
            .as_principal(&self.fixture.authenticate_specialist())
            .controls(BankReadControls::current(request_scope(), 16).unwrap())
            .execute()
            .expect("the assigned specialist should observe the death notice")
            .rows()[0]
            .death_notice()
            .status()
    }
}

pub fn dispatch_world(scenario: &str) -> DispatchWorld {
    let rail = spawn_rail();
    install_dispatch_world(scenario, rail)
}

pub fn dispatch_world_with_protocol_support(
    scenario: &str,
    protocol_support: RailProtocolSupportProfile,
) -> DispatchWorld {
    let rail = spawn_rail_with_protocol_support(protocol_support);
    install_dispatch_world(scenario, rail)
}

pub fn install_dispatch_world(scenario: &str, rail: RailProcessHandle) -> DispatchWorld {
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

pub fn rail_correlation(world: &DispatchWorld) -> RailCorrelation {
    world
        .transport
        .attempts()
        .into_iter()
        .next()
        .expect("a dispatched effect reaches the rail with one correlation")
}

pub fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("external-notification-{identity}")).unwrap()
}

#[path = "external_effect_dispatch/duplicate_acknowledgement.rs"]
pub mod duplicate_acknowledgement;
