//! The real rail sends immutable completion bytes after physical consequence.

pub use std::time::Duration;

pub use bank_external_rail::completion_wire::{
    COMPLETION_V1_MAGIC, CUSTODY_ACK_V1_BYTES, CUSTODY_ACK_V1_MAGIC, SIGNATURE_BYTES,
};
pub use bank_external_rail::test_control::{select_fault, FaultScript};
pub use bank_external_rail::{
    dispatch, inquire_completed_effect_count, inquire_completion_delivery_posture,
    RailCompletionDeliveryConfiguration, RailCorrelation, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailProcessHandle, RailServer,
};
pub use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
pub use sha2::{Digest, Sha256};
pub use tokio::io::{AsyncReadExt, AsyncWriteExt};
pub use tokio::net::{TcpListener, TcpStream};
pub use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub const TIMEOUT: Duration = Duration::from_secs(5);

pub fn notice_attempt() -> RailDispatch {
    let payload = [3u64, 12, 1]
        .into_iter()
        .flat_map(u64::to_be_bytes)
        .collect::<Vec<_>>();
    RailDispatch {
        correlation: RailCorrelation::new("estate-death-notice-rail", [4u8; 32]),
        payload: RailEffectPayload::new(
            "EstateDeathNotificationEffect",
            BoundaryProtocolIdentity::new("bank.estate.death-notification"),
            BoundaryProtocolVersion::new(1),
            24,
            payload,
        ),
    }
}

pub async fn read_http_body(stream: &mut TcpStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let header_end = loop {
        let mut chunk = [0u8; 512];
        let read = stream.read(&mut chunk).await.unwrap();
        assert!(read > 0);
        bytes.extend_from_slice(&chunk[..read]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let header = std::str::from_utf8(&bytes[..header_end]).unwrap();
    let length = header
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .and_then(|value| value.trim().parse::<usize>().ok())
        })
        .unwrap();
    while bytes.len() - header_end < length {
        let mut chunk = [0u8; 512];
        let read = stream.read(&mut chunk).await.unwrap();
        assert!(read > 0);
        bytes.extend_from_slice(&chunk[..read]);
    }
    bytes[header_end..header_end + length].to_vec()
}

pub async fn reply(stream: &mut TcpStream, body: &[u8]) {
    stream
        .write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .as_bytes(),
        )
        .await
        .unwrap();
    stream.write_all(body).await.unwrap();
}

pub async fn reply_chunked_oversized(stream: &mut TcpStream) {
    stream
        .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let first = vec![0u8; CUSTODY_ACK_V1_BYTES];
    let first_header = format!("{:x}\r\n", first.len());
    stream.write_all(first_header.as_bytes()).await.unwrap();
    stream.write_all(&first).await.unwrap();
    stream.write_all(b"\r\n1\r\nX\r\n0\r\n\r\n").await.unwrap();
}

pub fn assert_signed_meaning(body: &[u8], rail_key: &VerifyingKey) {
    assert!(body.starts_with(COMPLETION_V1_MAGIC));
    let signed_length = body.len() - SIGNATURE_BYTES;
    let signature = ed25519_dalek::Signature::from_bytes(body[signed_length..].try_into().unwrap());
    rail_key
        .verify_strict(&body[..signed_length], &signature)
        .unwrap();
    let mut cursor = 16;
    assert_eq!(take_text(body, &mut cursor), "bank-process-court");
    assert_eq!(take_text(body, &mut cursor), "rail-primary");
    assert_eq!(take(body, &mut cursor, 8), &1u64.to_be_bytes());
    take(body, &mut cursor, 32);
    let issued = u64::from_be_bytes(take(body, &mut cursor, 8).try_into().unwrap());
    let expiry = u64::from_be_bytes(take(body, &mut cursor, 8).try_into().unwrap());
    assert_eq!(expiry - issued, 60);
    assert_eq!(
        take_text(body, &mut cursor),
        "bank.estate.death-notification"
    );
    assert_eq!(take(body, &mut cursor, 2), &1u16.to_be_bytes());
    assert_eq!(take_text(body, &mut cursor), "estate-death-notice-rail");
    assert_eq!(take_short(body, &mut cursor), &[4u8; 32]);
    let payload_len = u32::from_be_bytes(take(body, &mut cursor, 4).try_into().unwrap()) as usize;
    let payload = take(body, &mut cursor, payload_len);
    assert_eq!(
        payload,
        [3u64, 12, 1]
            .into_iter()
            .flat_map(u64::to_be_bytes)
            .collect::<Vec<_>>()
    );
    assert_eq!(cursor, signed_length);
}

pub fn signed_ack(body: &[u8], bank_key: &SigningKey) -> Vec<u8> {
    signed_ack_with_posture(body, bank_key, 1)
}

pub fn signed_ack_with_posture(body: &[u8], bank_key: &SigningKey, posture: u8) -> Vec<u8> {
    let mut cursor = 16;
    take_short(body, &mut cursor);
    take_short(body, &mut cursor);
    take(body, &mut cursor, 8);
    let message_id = take(body, &mut cursor, 32);
    let mut acknowledgement = Vec::new();
    acknowledgement.extend_from_slice(CUSTODY_ACK_V1_MAGIC);
    acknowledgement.push(posture);
    acknowledgement.extend_from_slice(message_id);
    acknowledgement.extend_from_slice(&Sha256::digest(body));
    acknowledgement.extend_from_slice(&bank_key.sign(&acknowledgement).to_bytes());
    acknowledgement
}

pub fn take_text<'a>(bytes: &'a [u8], cursor: &mut usize) -> &'a str {
    std::str::from_utf8(take_short(bytes, cursor)).unwrap()
}

pub fn take_short<'a>(bytes: &'a [u8], cursor: &mut usize) -> &'a [u8] {
    let length = u16::from_be_bytes(take(bytes, cursor, 2).try_into().unwrap()) as usize;
    take(bytes, cursor, length)
}

pub fn take<'a>(bytes: &'a [u8], cursor: &mut usize, length: usize) -> &'a [u8] {
    let start = *cursor;
    *cursor += length;
    &bytes[start..*cursor]
}
