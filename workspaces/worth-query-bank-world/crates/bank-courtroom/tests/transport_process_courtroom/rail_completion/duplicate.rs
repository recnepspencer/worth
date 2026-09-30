//! Exact signed HTTP replays through the installed Bank callback route.

use std::net::SocketAddr;

use bank_external_rail::completion_wire::CUSTODY_ACK_V1_BYTES;
use ed25519_dalek::{Signature, VerifyingKey};
use sha2::{Digest, Sha256};

pub(super) async fn assert_exact_callbacks(
    client: &reqwest::Client,
    bank_address: SocketAddr,
    envelope: &[u8],
) {
    for _ in 0..100 {
        let response = client
            .post(format!("http://{bank_address}/v1/inbound/rail-completions"))
            .body(envelope.to_vec())
            .send()
            .await
            .expect("exact callback replay should reach Bank");
        assert_eq!(response.status(), reqwest::StatusCode::OK);
        let acknowledgement = response.bytes().await.unwrap();
        assert_eq!(acknowledgement.len(), CUSTODY_ACK_V1_BYTES);
        assert!(
            matches!(acknowledgement[16], 2 | 5),
            "duplicate returns performed or already-completed custody"
        );
    }
}

pub(super) fn assert_permanent_denial(
    acknowledgement: &[u8],
    original: &[u8],
    altered: &[u8],
    bank_public_key: [u8; 32],
) {
    assert_eq!(acknowledgement.len(), CUSTODY_ACK_V1_BYTES);
    assert_eq!(&acknowledgement[..16], b"BANK-CUSTODY-ACK");
    assert_eq!(
        acknowledgement[16], 4,
        "authoritative conflict is permanent"
    );
    let mut offset = 16;
    for _ in 0..2 {
        let length = u16::from_be_bytes(original[offset..offset + 2].try_into().unwrap()) as usize;
        offset += 2 + length;
    }
    offset += 8;
    assert_eq!(&acknowledgement[17..49], &original[offset..offset + 32]);
    assert_eq!(&acknowledgement[49..81], Sha256::digest(altered).as_slice());
    assert_ne!(
        &acknowledgement[49..81],
        Sha256::digest(original).as_slice()
    );
    let signature = Signature::from_bytes(acknowledgement[81..].try_into().unwrap());
    VerifyingKey::from_bytes(&bank_public_key)
        .unwrap()
        .verify_strict(&acknowledgement[..81], &signature)
        .unwrap();
}
