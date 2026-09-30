//! Shared real rail and callback installation for payment lifecycle courts.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use bank_domain::model::{AccountId, PaymentId};
use bank_external_rail::test_control::FaultScript;
use bank_external_rail::{RailCompletionDeliveryConfiguration, RailProcessHandle};
use bank_http_adapter::{
    BankHttpServerConfiguration, BankRailCallbackServer, BankRailCallbackServerBinding,
    BankRailCompletionServerInstallation,
};
use bank_server::{BankAuthenticatedPrincipal, BankIdentityRuntime};

use super::authentication;
use super::fixture::{
    ordinary_read_world_with_approval_authentication, APPROVER, OWNER, RECIPIENT,
};
use super::rail_transport::BankEstateRailTransport;
use super::{CallbackProxy, TIMEOUT};

const RAIL_PUBLIC_KEY: [u8; 32] = [
    234, 74, 108, 99, 226, 156, 82, 10, 190, 245, 80, 123, 19, 46, 197, 249, 149, 71, 118, 174,
    190, 190, 123, 146, 66, 30, 234, 105, 20, 70, 210, 44,
];
const BANK_PUBLIC_KEY: [u8; 32] = [
    253, 23, 36, 56, 90, 160, 199, 91, 100, 251, 120, 205, 96, 47, 161, 217, 145, 253, 235, 247,
    107, 19, 197, 142, 215, 2, 234, 200, 53, 233, 246, 24,
];
const AUDIENCE: &str = "bank-payment-process-court";

pub(super) struct PaymentCourt {
    pub rail: RailProcessHandle,
    pub runtime: Arc<BankIdentityRuntime>,
    pub transport: Arc<BankEstateRailTransport>,
    pub bank: BankRailCallbackServer,
    pub proxy: CallbackProxy,
    pub approver: BankAuthenticatedPrincipal,
    pub owner: BankAuthenticatedPrincipal,
    pub recipient: BankAuthenticatedPrincipal,
    pub payment_id: PaymentId,
    pub source_account: AccountId,
    pub destination_account: AccountId,
}

impl PaymentCourt {
    pub async fn start(scenario: &str) -> Self {
        let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
            Path::new(env!("CARGO_BIN_EXE_cold-bank-rail")),
            "127.0.0.1:0",
        )
        .expect("separate payment rail process should bind");
        assert_ne!(rail.pid(), std::process::id());

        let fixture = tokio::task::block_in_place(|| {
            ordinary_read_world_with_approval_authentication(
                scenario,
                authentication::approval_configuration(),
            )
        });
        let approver = tokio::task::block_in_place(|| fixture.authenticate(APPROVER));
        let owner = tokio::task::block_in_place(|| fixture.authenticate(OWNER));
        let recipient = tokio::task::block_in_place(|| fixture.authenticate(RECIPIENT));
        let payment_id = fixture.payment;
        let source_account = fixture.business_account;
        let destination_account = fixture.recipient_account;
        let runtime = fixture.into_shared_runtime();
        let transport = Arc::new(BankEstateRailTransport::connected_to(
            rail.local_addr(),
            rail.test_control_addr(),
        ));
        tokio::task::block_in_place(|| {
            runtime
                .install_external_effect_transport(transport.clone())
                .expect("payment rail transport installs");
        });

        let binding =
            BankRailCallbackServerBinding::bind(BankHttpServerConfiguration::local_ephemeral())
                .await
                .expect("callback HTTP should bind");
        let bank = binding
            .install_shared(
                Arc::clone(&runtime),
                BankRailCompletionServerInstallation::new(
                    RAIL_PUBLIC_KEY,
                    [9; 32],
                    AUDIENCE.to_owned(),
                    "rail-primary".to_owned(),
                    1,
                    60,
                )
                .expect("fixed rail verifier and ACK signer install"),
            )
            .expect("Bank callback HTTP should start");
        let proxy = CallbackProxy::start(bank.local_address()).await;
        rail.install_completion_delivery(
            &RailCompletionDeliveryConfiguration::new(
                format!("http://{}/v1/inbound/rail-completions", proxy.address),
                AUDIENCE.to_owned(),
                "rail-primary".to_owned(),
                1,
                [7; 32],
                BANK_PUBLIC_KEY,
                60,
                2,
                4,
                Duration::from_millis(50),
                TIMEOUT,
                None,
            )
            .expect("bounded payment callback sender configures"),
        )
        .expect("rail sender installs before dispatch");
        tokio::task::block_in_place(|| {
            transport.under(FaultScript::CommitThenLoseResponse, TIMEOUT);
        });
        Self {
            rail,
            runtime,
            transport,
            bank,
            proxy,
            approver,
            owner,
            recipient,
            payment_id,
            source_account,
            destination_account,
        }
    }
}
