//! Independent test mechanism for the installed Query verifier boundary.

use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

use super::schema::{PROTOCOL, SOURCE};
use crate::domain_computation::application_aftermath::{
    WorthQueryDispatchOutboxRecord, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial as Denial,
};

const AUDIENCE: &str = "inbound-test-audience";
const KEY: &[u8] = b"independent-inbound-test-secret-v1";
const BODY_PREFIX: usize = 1 + 8 + 8 + 32 + 32 + 2;
const MAC_BYTES: usize = 32;
const RIGHT_DOMAIN: u8 = 1;
const WRONG_DOMAIN: u8 = 2;

pub(super) struct TestVerifier;

impl WorthQueryInboundOccurrenceVerifier for TestVerifier {
    fn audience(&self) -> &str {
        AUDIENCE
    }

    fn source_identity(&self) -> &str {
        SOURCE
    }

    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        static IDENTITY: BoundaryProtocolIdentity = BoundaryProtocolIdentity::new(PROTOCOL);
        &IDENTITY
    }

    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }

    fn verify(
        &self,
        envelope: &[u8],
        now_unix_seconds: u64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, Denial> {
        if envelope.len() < BODY_PREFIX + MAC_BYTES {
            return Err(Denial::Malformed);
        }
        let body_len = envelope.len() - MAC_BYTES;
        let (body, signature) = envelope.split_at(body_len);
        if mac(body).as_slice() != signature {
            return Err(Denial::AuthenticationFailed);
        }
        let domain = match body[0] {
            RIGHT_DOMAIN => PROTOCOL,
            WRONG_DOMAIN => "inbound.test.wrong-domain",
            _ => return Err(Denial::UnsupportedVersion),
        };
        let issued_at = u64::from_be_bytes(body[1..9].try_into().unwrap());
        let expires_at = u64::from_be_bytes(body[9..17].try_into().unwrap());
        if issued_at > now_unix_seconds || expires_at < now_unix_seconds {
            return Err(Denial::Expired);
        }
        let message_identity = body[17..49].try_into().unwrap();
        let correlation_token = body[49..81].try_into().unwrap();
        let payload_len = usize::from(u16::from_be_bytes(body[81..83].try_into().unwrap()));
        if body.len() != BODY_PREFIX + payload_len {
            return Err(Denial::Malformed);
        }
        Ok(WorthQueryInboundOccurrenceClaims {
            audience: AUDIENCE.to_owned(),
            source_identity: SOURCE.to_owned(),
            key_epoch: 1,
            message_identity,
            signed_meaning_digest: Sha256::digest(body).into(),
            issued_at_unix_seconds: issued_at,
            expires_at_unix_seconds: expires_at,
            protocol_identity: BoundaryProtocolIdentity::new(domain),
            protocol_version: BoundaryProtocolVersion::new(1),
            correlation_family: "inbound-test-family".to_owned(),
            correlation_token,
            payload: body[BODY_PREFIX..].to_vec(),
        })
    }
}

pub(super) fn signed_envelope(
    record: &WorthQueryDispatchOutboxRecord,
    message_identity: [u8; 32],
    payload: &[u8],
    wrong_domain: bool,
) -> Vec<u8> {
    signed_envelope_for_seconds(record, message_identity, payload, wrong_domain, 30)
}

pub(super) fn signed_envelope_for_seconds(
    record: &WorthQueryDispatchOutboxRecord,
    message_identity: [u8; 32],
    payload: &[u8],
    wrong_domain: bool,
    validity_seconds: u64,
) -> Vec<u8> {
    signed_envelope_for_token(
        *record.correlation().bytes(),
        message_identity,
        payload,
        wrong_domain,
        validity_seconds,
    )
}

pub(super) fn signed_envelope_for_token(
    correlation_token: [u8; 32],
    message_identity: [u8; 32],
    payload: &[u8],
    wrong_domain: bool,
    validity_seconds: u64,
) -> Vec<u8> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let mut body = Vec::with_capacity(BODY_PREFIX + payload.len() + MAC_BYTES);
    body.push(if wrong_domain {
        WRONG_DOMAIN
    } else {
        RIGHT_DOMAIN
    });
    body.extend_from_slice(&(now - 1).to_be_bytes());
    body.extend_from_slice(&(now + validity_seconds).to_be_bytes());
    body.extend_from_slice(&message_identity);
    body.extend_from_slice(&correlation_token);
    body.extend_from_slice(&u16::try_from(payload.len()).unwrap().to_be_bytes());
    body.extend_from_slice(payload);
    let signature = mac(&body);
    body.extend_from_slice(&signature);
    body
}

fn mac(body: &[u8]) -> [u8; MAC_BYTES] {
    let mut inner_pad = [0x36; 64];
    let mut outer_pad = [0x5c; 64];
    for (index, byte) in KEY.iter().enumerate() {
        inner_pad[index] ^= byte;
        outer_pad[index] ^= byte;
    }
    let mut inner = Sha256::new();
    inner.update(inner_pad);
    inner.update(body);
    let inner_digest = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(outer_pad);
    outer.update(inner_digest);
    outer.finalize().into()
}
