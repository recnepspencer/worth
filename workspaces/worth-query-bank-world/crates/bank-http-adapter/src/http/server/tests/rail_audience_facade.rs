//! Public Bank audience path: payment wait declaration and installed callback.

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use bank_domain::model::AccountId;
use bank_domain::schema::approved_payment_inbound_binding;
use bank_server::approved_business_payment_definition;
use ed25519_dalek::{Signer, SigningKey};
use worth_query_host::facade::declaration::application_program::ApplicationWorkflowNodeKind;

use super::super::inbound_completion::{
    BankRailCallbackServerBinding, BankRailCloseAssessment, BankRailCompletionServerInstallation,
};
use super::super::BankHttpServerConfiguration;
use super::fixture::application;

#[tokio::test]
async fn public_payment_wait_and_callback_installation_refuse_unowned_completion() {
    let definition = approved_business_payment_definition().unwrap();
    assert!(definition
        .nodes()
        .iter()
        .any(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::AwaitInbound(_))));
    let _declared_payment_contract = approved_payment_inbound_binding();

    let runtime = Arc::new(application(AccountId::new(100).unwrap()).runtime);
    let binding =
        BankRailCallbackServerBinding::bind(BankHttpServerConfiguration::local_ephemeral())
            .await
            .unwrap();
    let server = binding
        .install_shared(
            Arc::clone(&runtime),
            BankRailCompletionServerInstallation::new(
                SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes(),
                [9; 32],
                "bank-process-court".into(),
                "rail-primary".into(),
                1,
                0,
            )
            .unwrap(),
        )
        .unwrap();
    let mut envelope = include_str!("../../protocol/inbound_payment_completion_v1.hex")
        .trim()
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    envelope[90..98].copy_from_slice(&now.to_be_bytes());
    envelope[98..106].copy_from_slice(&(now + 60).to_be_bytes());
    let signed = envelope.len() - 64;
    let signature = SigningKey::from_bytes(&[7; 32]).sign(&envelope[..signed]);
    envelope[signed..].copy_from_slice(&signature.to_bytes());

    let response = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/inbound/rail-completions",
            server.local_address()
        ))
        .body(envelope)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::CONFLICT);
    assert!(response.bytes().await.unwrap().is_empty(), "no custody ACK");
    let close = server.shutdown().await.unwrap();
    assert_eq!(
        close.into_continuation().continue_once().await,
        BankRailCloseAssessment::Assessed {
            known_remaining: 0,
            outstanding_dispatches: 0,
            retained_accepted_occurrences: 0,
            blocked: 0,
            unavailable_routes: 0,
            next_expiry_unix_seconds: None,
        }
    );
}
