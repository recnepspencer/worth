//! An installed Bank route rejects a signed incompatible version before custody.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bank_domain::model::AccountId;
use ed25519_dalek::{Signer, SigningKey};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial, WorthQueryInboundVerificationDenial,
};

use super::super::inbound_completion::BankRailCompletionServerInstallation;
use super::*;

#[tokio::test]
async fn valid_signature_on_unsupported_version_denies_before_owner_custody() {
    let application = Arc::new(application(AccountId::new(100).unwrap()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let installation = BankRailCompletionServerInstallation::new(
        SigningKey::from_bytes(&[7; 32]).verifying_key().to_bytes(),
        [9; 32],
        "bank-process-court".into(),
        "rail-primary".into(),
        1,
        0,
    )
    .unwrap();
    let server = super::super::bind_application_to_listener(
        application,
        listener,
        BankHttpServerConfiguration::local_ephemeral(),
        Some(installation),
    )
    .unwrap();
    let mut envelope = include_str!("../../protocol/inbound_completion_v1.hex")
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
    let v1_signature = SigningKey::from_bytes(&[7; 32]).sign(&envelope[..signed]);
    envelope[signed..].copy_from_slice(&v1_signature.to_bytes());
    let v1_twin = envelope.clone();
    let protocol = b"bank.estate.death-notification";
    let version = envelope
        .windows(protocol.len())
        .position(|window| window == protocol)
        .unwrap()
        + protocol.len();
    assert_eq!(&envelope[version..version + 2], &[0, 1]);
    envelope[version..version + 2].copy_from_slice(&2u16.to_be_bytes());
    let signature = SigningKey::from_bytes(&[7; 32]).sign(&envelope[..signed]);
    envelope[signed..].copy_from_slice(&signature.to_bytes());

    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(5),
        cancellation.token(),
    );
    let route = server.rail_completion.as_ref().unwrap();
    let before = server.observe_estate_rail_completion_cost().unwrap();
    assert!(matches!(
        route.receive_and_sign(&envelope, &request),
        Err(WorthQueryInboundAdmissionDenial::Verification(
            WorthQueryInboundVerificationDenial::UnsupportedVersion
        ))
    ));
    assert!(server.observe_rail_completion([4; 32]).is_none());
    let after_v2 = server.observe_estate_rail_completion_cost().unwrap();
    assert_eq!(after_v2.terminal_key_probes(), before.terminal_key_probes());
    assert_eq!(after_v2.outbox_key_probes(), before.outbox_key_probes());
    assert_eq!(after_v2.custody_key_probes(), before.custody_key_probes());
    assert_eq!(
        after_v2.world_publication_attempts(),
        before.world_publication_attempts()
    );

    let mut foreign_audience = v1_twin.clone();
    let audience = b"bank-process-court";
    let audience_offset = foreign_audience
        .windows(audience.len())
        .position(|window| window == audience)
        .unwrap();
    foreign_audience[audience_offset..audience_offset + audience.len()]
        .copy_from_slice(b"bank-foreign-court");
    let signature = SigningKey::from_bytes(&[7; 32]).sign(&foreign_audience[..signed]);
    foreign_audience[signed..].copy_from_slice(&signature.to_bytes());
    let mut expired = v1_twin.clone();
    expired[98..106].copy_from_slice(&now.saturating_sub(1).to_be_bytes());
    let signature = SigningKey::from_bytes(&[7; 32]).sign(&expired[..signed]);
    expired[signed..].copy_from_slice(&signature.to_bytes());
    for (body, expected) in [
        (
            foreign_audience,
            WorthQueryInboundVerificationDenial::AuthenticationFailed,
        ),
        (expired, WorthQueryInboundVerificationDenial::Expired),
    ] {
        assert!(matches!(
            route.receive_and_sign(&body, &request),
            Err(WorthQueryInboundAdmissionDenial::Verification(denial)) if denial == expected
        ));
        let response = reqwest::Client::new()
            .post(format!(
                "http://{}/v1/inbound/rail-completions",
                server.local_address()
            ))
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
        assert!(response.bytes().await.unwrap().is_empty(), "no custody ACK");
    }
    let after_auth_denials = server.observe_estate_rail_completion_cost().unwrap();
    assert_eq!(
        after_auth_denials.terminal_key_probes(),
        before.terminal_key_probes()
    );
    assert_eq!(
        after_auth_denials.outbox_key_probes(),
        before.outbox_key_probes()
    );
    assert_eq!(
        after_auth_denials.custody_key_probes(),
        before.custody_key_probes()
    );

    assert!(matches!(
        route.receive_and_sign(&v1_twin, &request),
        Err(WorthQueryInboundAdmissionDenial::UnknownCorrelation)
    ));
    let after_v1 = server.observe_estate_rail_completion_cost().unwrap();
    assert!(after_v1.terminal_key_probes() > after_v2.terminal_key_probes());

    let response = reqwest::Client::new()
        .post(format!(
            "http://{}/v1/inbound/rail-completions",
            server.local_address()
        ))
        .body(envelope)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), reqwest::StatusCode::UNAUTHORIZED);
    assert!(response.bytes().await.unwrap().is_empty(), "no custody ACK");

    // Keep the declared body incomplete. A response proves the 4 KiB extractor
    // limit rejected before buffering the server-wide 64 KiB allowance.
    let mut oversized = tokio::net::TcpStream::connect(server.local_address())
        .await
        .unwrap();
    oversized
        .write_all(
            b"POST /v1/inbound/rail-completions HTTP/1.1\r\nHost: localhost\r\nContent-Length: 4098\r\n\r\n",
        )
        .await
        .unwrap();
    oversized.write_all(&[0; 4097]).await.unwrap();
    let mut head = [0; 256];
    let count = tokio::time::timeout(Duration::from_secs(2), oversized.read(&mut head))
        .await
        .expect("oversized incomplete body is refused before its final byte")
        .unwrap();
    assert!(
        std::str::from_utf8(&head[..count])
            .unwrap()
            .starts_with("HTTP/1.1 413"),
        "the callback extractor enforces the 4 KiB limit"
    );
    let close = server.shutdown().await.unwrap();
    assert!(close.rail_completion().is_some());
}
