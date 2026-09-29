//! The real rail sends immutable completion bytes after physical consequence.

use std::time::Duration;

use bank_external_rail::completion_wire::{
    COMPLETION_V1_MAGIC, CUSTODY_ACK_V1_BYTES, CUSTODY_ACK_V1_MAGIC, SIGNATURE_BYTES,
};
use bank_external_rail::test_control::{select_fault, FaultScript};
use bank_external_rail::{
    dispatch, inquire_completed_effect_count, inquire_completion_delivery_posture,
    RailCompletionDeliveryConfiguration, RailCorrelation, RailDispatch, RailEffectPayload,
    RailExchangeOutcome, RailProcessHandle, RailServer,
};
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

const TIMEOUT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn separate_rail_process_installs_sender_before_dispatch_and_reports_exhaustion() {
    let mut rail = RailProcessHandle::spawn_awaiting_completion_installation(
        env!("CARGO_BIN_EXE_bank-external-rail"),
        "127.0.0.1:0",
    )
    .unwrap();
    let config = RailCompletionDeliveryConfiguration::new(
        "http://127.0.0.1:1/v1/inbound/rail-completions".into(),
        "bank-process-court".into(),
        "rail-primary".into(),
        1,
        [7; 32],
        SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
        60,
        2,
        1,
        Duration::from_millis(20),
        Duration::from_millis(100),
        None,
    )
    .unwrap();
    rail.install_completion_delivery(&config).unwrap();
    let attempt = notice_attempt();
    assert_eq!(
        dispatch(rail.local_addr(), attempt, TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    for _ in 0..30 {
        let posture = inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
            .await
            .unwrap();
        if posture.exhausted == 1 {
            assert_eq!(posture.pending, 1, "exhaustion retains sender obligation");
            assert_eq!(
                inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap(),
                1
            );
            select_fault(
                rail.test_control_addr(),
                FaultScript::AcknowledgeWithoutCompleting,
                TIMEOUT,
            )
            .await
            .unwrap();
            let mut acknowledged_only = notice_attempt();
            acknowledged_only.correlation =
                RailCorrelation::new("estate-death-notice-rail", [5u8; 32]);
            assert_eq!(
                dispatch(rail.local_addr(), acknowledged_only, TIMEOUT).await,
                RailExchangeOutcome::Acknowledged
            );
            assert_eq!(
                inquire_completion_delivery_posture(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap()
                    .pending,
                1
            );
            assert_eq!(
                inquire_completed_effect_count(rail.local_addr(), TIMEOUT)
                    .await
                    .unwrap(),
                1
            );
            let closed = rail
                .close()
                .expect("orderly process close reports sender obligations");
            assert_eq!(closed.pending, 1);
            assert_eq!(closed.exhausted, 1);
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("rail process did not expose unresolved sender exhaustion");
}

#[tokio::test]
async fn completed_owner_retries_identical_signed_bytes_until_exact_bank_ack() {
    let bank = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bank_address = bank.local_addr().unwrap();
    let rail_signing = SigningKey::from_bytes(&[7; 32]);
    let bank_signing = SigningKey::from_bytes(&[9; 32]);
    let bank_verifying_key = bank_signing.verifying_key().to_bytes();
    let bank_task = tokio::spawn(async move {
        let mut messages = Vec::new();
        for attempt in 0..3 {
            let (mut connection, _) = bank.accept().await.unwrap();
            let message = read_http_body(&mut connection).await;
            assert_signed_meaning(&message, &rail_signing.verifying_key());
            if attempt == 0 {
                reply_chunked_oversized(&mut connection).await;
            } else if attempt == 1 {
                reply(&mut connection, b"unsigned acknowledgement").await;
            } else {
                let acknowledgement = signed_ack(&message, &bank_signing);
                reply(&mut connection, &acknowledgement).await;
            }
            messages.push(message);
        }
        messages
    });

    let mut rail = RailServer::bind("127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let rail_address = rail.local_addr().unwrap();
    rail.install_completion_delivery(
        RailCompletionDeliveryConfiguration::new(
            format!("http://{bank_address}/v1/inbound/rail-completions"),
            "bank-process-court".into(),
            "rail-primary".into(),
            1,
            [7; 32],
            bank_verifying_key,
            60,
            2,
            3,
            Duration::from_millis(20),
            Duration::from_secs(1),
            None,
        )
        .unwrap(),
    )
    .unwrap();
    let rail_task = tokio::spawn(rail.serve());
    let attempt = notice_attempt();
    assert_eq!(
        dispatch(rail_address, attempt.clone(), TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    assert_eq!(
        dispatch(rail_address, attempt, TIMEOUT).await,
        RailExchangeOutcome::Completed
    );
    let messages = tokio::time::timeout(TIMEOUT, bank_task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        messages[0], messages[1],
        "ambiguous ACK resends identical bytes and ID"
    );
    assert_eq!(messages[1], messages[2]);
    assert_eq!(
        inquire_completed_effect_count(rail_address, TIMEOUT)
            .await
            .unwrap(),
        1
    );
    for _ in 0..30 {
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
    panic!("authenticated custody ACK did not release the rail obligation");
}

fn notice_attempt() -> RailDispatch {
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

async fn read_http_body(stream: &mut TcpStream) -> Vec<u8> {
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

async fn reply(stream: &mut TcpStream, body: &[u8]) {
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

async fn reply_chunked_oversized(stream: &mut TcpStream) {
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

fn assert_signed_meaning(body: &[u8], rail_key: &VerifyingKey) {
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

fn signed_ack(body: &[u8], bank_key: &SigningKey) -> Vec<u8> {
    let mut cursor = 16;
    take_short(body, &mut cursor);
    take_short(body, &mut cursor);
    take(body, &mut cursor, 8);
    let message_id = take(body, &mut cursor, 32);
    let mut acknowledgement = Vec::new();
    acknowledgement.extend_from_slice(CUSTODY_ACK_V1_MAGIC);
    acknowledgement.push(1);
    acknowledgement.extend_from_slice(message_id);
    acknowledgement.extend_from_slice(&Sha256::digest(body));
    acknowledgement.extend_from_slice(&bank_key.sign(&acknowledgement).to_bytes());
    acknowledgement
}

fn take_text<'a>(bytes: &'a [u8], cursor: &mut usize) -> &'a str {
    std::str::from_utf8(take_short(bytes, cursor)).unwrap()
}

fn take_short<'a>(bytes: &'a [u8], cursor: &mut usize) -> &'a [u8] {
    let length = u16::from_be_bytes(take(bytes, cursor, 2).try_into().unwrap()) as usize;
    take(bytes, cursor, length)
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, length: usize) -> &'a [u8] {
    let start = *cursor;
    *cursor += length;
    &bytes[start..*cursor]
}
