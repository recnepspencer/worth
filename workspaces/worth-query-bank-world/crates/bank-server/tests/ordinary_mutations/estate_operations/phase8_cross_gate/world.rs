//! Cross-gate world construction (R8.65) for Gate 8.3 integration evidence.

use std::sync::Arc;
use std::time::Duration;

use bank_domain::{estate::DeathNoticeStatus, proposals::BankIdempotencyKey};
use bank_external_rail::RailProcessHandle;
use bank_server::{queries, BankCommitReceipt, BankReadControls};

use super::super::external_effect_dispatch::rail_transport::{spawn_rail, BankEstateRailTransport};
use super::super::notify_death::fixture::{
    notification_world_with_clock_and_grant_validity, NotificationFixture,
};
use crate::authorization_time::AuthorizationTimeController;
use crate::support::request_scope;

pub(crate) const PATIENT: Duration = Duration::from_secs(5);

pub(crate) struct CrossGateWorld {
    pub(crate) fixture: NotificationFixture,
    pub(crate) transport: Arc<BankEstateRailTransport>,
    _rail: RailProcessHandle,
}

pub(crate) fn cross_gate_world(scenario: &str) -> CrossGateWorld {
    cross_gate_world_with_authorization_time(scenario, None)
}

pub(crate) fn cross_gate_world_with_authorization_time(
    scenario: &str,
    authorization_time: Option<AuthorizationTimeController>,
) -> CrossGateWorld {
    cross_gate_world_with_clock_and_grant_validity(scenario, authorization_time, None)
}

pub(crate) fn cross_gate_world_with_clock_and_grant_validity(
    scenario: &str,
    authorization_time: Option<AuthorizationTimeController>,
    grant_valid_until_epoch: Option<u64>,
) -> CrossGateWorld {
    let rail = spawn_rail();
    let transport = Arc::new(BankEstateRailTransport::connected_to(
        rail.local_addr(),
        rail.test_control_addr(),
    ));
    let fixture = notification_world_with_clock_and_grant_validity(
        &format!("phase8-cross-gate-{scenario}"),
        DeathNoticeStatus::Reported,
        authorization_time,
        grant_valid_until_epoch,
    );
    fixture
        .world
        .runtime
        .install_external_effect_transport(transport.clone())
        .expect("rail transport installs once per runtime");
    CrossGateWorld {
        fixture,
        transport,
        _rail: rail,
    }
}

impl CrossGateWorld {
    pub(crate) fn commit_notification(&self, identity: u8) -> BankCommitReceipt {
        self.commit_with(idempotency(identity))
    }

    pub(crate) fn commit_second_notification(&self, identity: u8) -> BankCommitReceipt {
        let outcome = {
            let source_actor = &self.fixture.authenticate_specialist();
            let source_scope = &request_scope();
            let source_key = &idempotency(identity);
            let source_intent = bank_domain::schema::NotifyEstateDeath::try_from(
                bank_domain::estate::EstateAction::NotifyDeath {
                    estate: self.fixture.second_estate,
                    notice: self.fixture.second_notice,
                    subject: self.fixture.other_subject,
                },
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
            assert!(matches!(
                source_completed.evidence(),
                worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
            ));
            let closed_source = source_completed
                .close_with_evidence(&source_request)
                .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
            bank_server::BankCommitReceipt::from_closed_source(closed_source)
        };
        let receipt = outcome;
        receipt
    }

    pub(crate) fn commit_with(&self, binding: BankIdempotencyKey) -> BankCommitReceipt {
        let outcome = {
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
            assert!(matches!(
                source_completed.evidence(),
                worth_query_host::facade::application_entry::WorthQuerySourceCompletion::Fresh { .. }
            ));
            let closed_source = source_completed
                .close_with_evidence(&source_request)
                .unwrap_or_else(|(_, cause)| panic!("lawful native source close: {cause:?}"));
            bank_server::BankCommitReceipt::from_closed_source(closed_source)
        };
        let receipt = outcome;
        receipt
    }

    /// Independent balance oracle — accounting revision of the estate account.
    pub(crate) fn estate_account_revision(&self) -> u64 {
        self.fixture
            .world
            .runtime
            .query(queries::account_summary(self.fixture.estate_account))
            .as_principal(&self.fixture.authenticate_deceased())
            .controls(BankReadControls::current(request_scope(), 1, 1_024).unwrap())
            .execute()
            .expect("estate account owner reads authoritative revision")
            .rows()[0]
            .accounting_revision()
            .get()
    }

    pub(crate) fn notice_status(&self) -> DeathNoticeStatus {
        self.fixture
            .world
            .runtime
            .query(queries::estate_case(self.fixture.estate))
            .as_principal(&self.fixture.authenticate_specialist())
            .controls(BankReadControls::current(request_scope(), 16, 20_000).unwrap())
            .execute()
            .expect("assigned specialist observes the authoritative death notice")
            .rows()[0]
            .death_notice()
            .status()
    }

    pub(crate) fn specialist_action(&self) -> bank_domain::estate::EstateAction {
        self.fixture
            .action(self.fixture.notice, self.fixture.deceased)
    }

    /// The action behind [`Self::commit_second_notification`] — a different
    /// estate, so `NotificationFixture::action` (which pins its own estate)
    /// cannot build it.
    /// Open recovery through the production Bank assembly path (R8.31 / §10.4).
    pub(crate) fn open_recovery(
        &self,
        receipt: &BankCommitReceipt,
    ) -> bank_server::BankCommitRecoveryHandle {
        self.fixture
            .world
            .runtime
            .open_commit_recovery(receipt)
            .expect("production mint")
    }
}

pub(crate) fn idempotency(identity: u8) -> BankIdempotencyKey {
    BankIdempotencyKey::new(format!("phase8-notification-{identity}")).unwrap()
}
