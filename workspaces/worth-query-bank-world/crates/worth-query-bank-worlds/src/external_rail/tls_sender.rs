//! The rail's HTTPS peer trust uses only the installed Bank certificate.

pub use std::sync::Arc;
pub use std::time::Duration;

pub use bank_external_rail::{
    dispatch, inquire_completion_delivery_posture, RailCompletionDeliveryConfiguration,
    RailCompletionDeliveryConfigurationDenial, RailCorrelation, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailServer,
};
pub use ed25519_dalek::{Signer, SigningKey};
pub use rcgen::generate_simple_self_signed;
pub use sha2::{Digest, Sha256};
pub use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
pub use tokio::net::TcpListener;
pub use tokio_rustls::rustls::{pki_types::PrivatePkcs8KeyDer, ServerConfig};
pub use tokio_rustls::TlsAcceptor;
pub use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub const TIMEOUT: Duration = Duration::from_secs(5);

pub fn configuration(
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

pub fn notice_attempt() -> RailDispatch {
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

pub async fn read_http_body(stream: &mut (impl AsyncRead + Unpin)) -> Vec<u8> {
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

pub fn signed_ack(body: &[u8], signer: &SigningKey) -> Vec<u8> {
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
