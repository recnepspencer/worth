//! The rail's HTTPS peer trust uses only the installed Bank certificate.

use std::sync::Arc;
use std::time::Duration;

use bank_external_rail::{
    dispatch, inquire_completion_delivery_posture, RailCompletionDeliveryConfiguration,
    RailCompletionDeliveryConfigurationDenial, RailCorrelation, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailServer,
};
use ed25519_dalek::{Signer, SigningKey};
use rcgen::generate_simple_self_signed;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio_rustls::rustls::{pki_types::PrivatePkcs8KeyDer, ServerConfig};
use tokio_rustls::TlsAcceptor;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

const TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn only_the_installed_bank_tls_root_can_receive_signed_completion() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let trusted_pem = certified.cert.pem();
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certified.cert.der().clone()],
            PrivatePkcs8KeyDer::from(certified.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let destination = format!(
        "https://localhost:{}/v1/inbound/rail-completions",
        address.port()
    );
    assert!(matches!(
        configuration(destination.clone(), None),
        Err(RailCompletionDeliveryConfigurationDenial::MissingPeerTrust)
    ));
    assert!(matches!(
        configuration(destination.clone(), Some("invalid PEM".to_owned())),
        Err(RailCompletionDeliveryConfigurationDenial::InvalidPeerTrust)
    ));
    let bank_signer = SigningKey::from_bytes(&[9; 32]);
    let bank = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut stream = acceptor
            .accept(tcp)
            .await
            .expect("installed root authenticates TLS");
        let body = read_http_body(&mut stream).await;
        let acknowledgement = signed_ack(&body, &bank_signer);
        stream
            .write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    acknowledgement.len(),
                )
                .as_bytes(),
            )
            .await
            .unwrap();
        stream.write_all(&acknowledgement).await.unwrap();
        body
    });
    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(
        configuration(destination.clone(), Some(trusted_pem)).unwrap(),
    )
    .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    assert_eq!(
        dispatch(rail_address, notice_attempt(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    let body = tokio::time::timeout(TIMEOUT, bank).await.unwrap().unwrap();
    assert!(body.starts_with(b"BANK-COMPLETION1"));
    for _ in 0..50 {
        if inquire_completion_delivery_posture(rail_address, TIMEOUT)
            .await
            .unwrap()
            .pending
            == 0
        {
            rail_task.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    rail_task.abort();
    panic!("pinned Bank TLS and signed ACK did not settle the sender");
}

#[tokio::test]
async fn foreign_tls_root_cannot_release_the_rail_sender_obligation() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
    let certified = generate_simple_self_signed(vec!["localhost".to_owned()]).unwrap();
    let foreign_pem = generate_simple_self_signed(vec!["localhost".to_owned()])
        .unwrap()
        .cert
        .pem();
    let server = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certified.cert.der().clone()],
            PrivatePkcs8KeyDer::from(certified.signing_key.serialize_der()).into(),
        )
        .unwrap();
    let acceptor = TlsAcceptor::from(Arc::new(server));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let destination = format!(
        "https://localhost:{}/v1/inbound/rail-completions",
        listener.local_addr().unwrap().port()
    );
    let bank = tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        acceptor.accept(tcp).await.is_err()
    });
    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(configuration(destination, Some(foreign_pem)).unwrap())
        .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    assert_eq!(
        dispatch(rail_address, notice_attempt(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    assert!(
        tokio::time::timeout(TIMEOUT, bank).await.unwrap().unwrap(),
        "foreign trust must fail the TLS handshake before any HTTP callback"
    );
    for _ in 0..50 {
        let posture = inquire_completion_delivery_posture(rail_address, TIMEOUT)
            .await
            .unwrap();
        if posture.exhausted == 1 {
            assert_eq!(posture.pending, 1);
            rail_task.abort();
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    rail_task.abort();
    panic!("foreign Bank certificate did not leave an exhausted sender obligation");
}

fn configuration(
    destination: String,
    pinned_pem: Option<String>,
) -> Result<RailCompletionDeliveryConfiguration, RailCompletionDeliveryConfigurationDenial> {
    RailCompletionDeliveryConfiguration::new(
        destination,
        "bank-process-court".into(),
        "rail-primary".into(),
        1,
        [7; 32],
        SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
        60,
        2,
        1,
        Duration::from_millis(20),
        Duration::from_secs(1),
        pinned_pem,
    )
}

fn notice_attempt() -> RailDispatch {
    RailDispatch {
        correlation: RailCorrelation::new("estate-death-notice-rail", [4; 32]),
        payload: RailEffectPayload::new(
            "EstateDeathNotificationEffect",
            BoundaryProtocolIdentity::new("bank.estate.death-notification"),
            BoundaryProtocolVersion::new(1),
            24,
            [3u64, 12, 1]
                .into_iter()
                .flat_map(u64::to_be_bytes)
                .collect::<Vec<_>>(),
        ),
    }
}

async fn read_http_body(stream: &mut (impl AsyncRead + Unpin)) -> Vec<u8> {
    let mut bytes = Vec::new();
    let boundary = loop {
        let mut chunk = [0u8; 1024];
        let read = stream.read(&mut chunk).await.unwrap();
        assert!(read > 0 && bytes.len() + read <= 8192);
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(offset) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
            break offset + 4;
        }
    };
    let headers = std::str::from_utf8(&bytes[..boundary]).unwrap();
    let length: usize = headers
        .lines()
        .find_map(|line| {
            line.split_once(':').and_then(|(name, value)| {
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse().unwrap())
            })
        })
        .unwrap();
    while bytes.len() - boundary < length {
        let mut chunk = [0u8; 1024];
        let read = stream.read(&mut chunk).await.unwrap();
        assert!(read > 0 && bytes.len() + read <= 8192);
        bytes.extend_from_slice(&chunk[..read]);
    }
    bytes[boundary..boundary + length].to_vec()
}

fn signed_ack(body: &[u8], signer: &SigningKey) -> Vec<u8> {
    let mut offset = 16;
    for _ in 0..2 {
        let length = u16::from_be_bytes(body[offset..offset + 2].try_into().unwrap()) as usize;
        offset += 2 + length;
    }
    offset += 8;
    let message_id = &body[offset..offset + 32];
    let mut ack = Vec::new();
    ack.extend_from_slice(b"BANK-CUSTODY-ACK");
    ack.push(1);
    ack.extend_from_slice(message_id);
    ack.extend_from_slice(&Sha256::digest(body));
    ack.extend_from_slice(&signer.sign(&ack).to_bytes());
    ack
}
