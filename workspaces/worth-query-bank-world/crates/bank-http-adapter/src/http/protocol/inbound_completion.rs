//! Bank-owned decoder for the rail's signed completion v1 wire contract.
//!
//! The rail encoder is deliberately not imported here. The signed body is:
//! `BANK-COMPLETION1`, length-prefixed audience and source, key epoch,
//! message ID, issue/expiry seconds, protocol identity/version, correlation
//! family/token, payload, then a detached Ed25519 signature.

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};

const PREFIX: &[u8; 16] = b"BANK-COMPLETION1";
const SIGNATURE_BYTES: usize = 64;
const ACK_PREFIX: &[u8; 16] = b"BANK-CUSTODY-ACK";
pub(crate) const MAXIMUM_COMPLETION_BYTES: usize = 4_096;

/// An untrusted wire selector. Query still authenticates the exact bytes
/// against the fixed verifier installed for the selected operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RailCompletionProtocol {
    EstateDeathNotice,
    ApprovedPaymentSettlement,
}

pub(crate) fn completion_protocol_for_selection(
    bytes: &[u8],
) -> Result<RailCompletionProtocol, CompletionWireDenial> {
    if bytes.len() > MAXIMUM_COMPLETION_BYTES {
        return Err(CompletionWireDenial::Oversized);
    }
    let signed_len = bytes
        .len()
        .checked_sub(SIGNATURE_BYTES)
        .ok_or(CompletionWireDenial::Malformed)?;
    let mut reader = Reader::new(&bytes[..signed_len]);
    if reader.take(PREFIX.len())? != PREFIX {
        return Err(CompletionWireDenial::Malformed);
    }
    reader.bytes_u16()?; // audience
    reader.bytes_u16()?; // source
    reader.take(8 + 32 + 8 + 8)?; // epoch, message, issue and expiry
    match std::str::from_utf8(reader.bytes_u16()?).map_err(|_| CompletionWireDenial::Malformed)? {
        "bank.estate.death-notification" => Ok(RailCompletionProtocol::EstateDeathNotice),
        "bank.payment.approved-settlement" => Ok(RailCompletionProtocol::ApprovedPaymentSettlement),
        _ => Err(CompletionWireDenial::Malformed),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VerifiedRailCompletion {
    pub audience: String,
    pub source: String,
    pub key_epoch: u64,
    pub message_id: [u8; 32],
    pub signed_meaning_digest: [u8; 32],
    pub issued_at_seconds: u64,
    pub expires_at_seconds: u64,
    pub protocol_identity: String,
    pub protocol_version: u16,
    pub correlation_family: String,
    pub correlation_token: Vec<u8>,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionWireDenial {
    Oversized,
    Malformed,
    BadSignature,
    WrongSource,
    Expired,
}

pub(crate) struct RailCompletionVerification<'a> {
    pub key: &'a VerifyingKey,
    pub audience: &'a str,
    pub source: &'a str,
    pub key_epoch: u64,
    pub now_seconds: u64,
    pub maximum_clock_skew_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BankCustodyAckPosture {
    AcceptedPending = 1,
    Performed = 2,
    AlreadyAccepted = 3,
    PermanentDenied = 4,
    AlreadyCompleted = 5,
}

/// A custody response is issued only after Query has retained or rejected
/// the exact occurrence. A transport success alone must never call this.
pub(crate) fn sign_custody_ack(
    envelope_digest: &[u8; 32],
    message_id: [u8; 32],
    posture: BankCustodyAckPosture,
    key: &SigningKey,
) -> Vec<u8> {
    let mut body = Vec::with_capacity(16 + 1 + 32 + 32 + SIGNATURE_BYTES);
    body.extend_from_slice(ACK_PREFIX);
    body.push(posture as u8);
    body.extend_from_slice(&message_id);
    body.extend_from_slice(envelope_digest);
    let signature = key.sign(&body);
    body.extend_from_slice(&signature.to_bytes());
    body
}

pub(crate) fn verify_completion(
    bytes: &[u8],
    expected: RailCompletionVerification<'_>,
) -> Result<VerifiedRailCompletion, CompletionWireDenial> {
    if bytes.len() > MAXIMUM_COMPLETION_BYTES {
        return Err(CompletionWireDenial::Oversized);
    }
    let signed_len = bytes
        .len()
        .checked_sub(SIGNATURE_BYTES)
        .ok_or(CompletionWireDenial::Malformed)?;
    let (signed, signature) = bytes.split_at(signed_len);
    let signature: [u8; SIGNATURE_BYTES] = signature
        .try_into()
        .map_err(|_| CompletionWireDenial::Malformed)?;
    expected
        .key
        .verify_strict(signed, &Signature::from_bytes(&signature))
        .map_err(|_| CompletionWireDenial::BadSignature)?;

    let mut reader = Reader::new(signed);
    if reader.take(PREFIX.len())? != PREFIX {
        return Err(CompletionWireDenial::Malformed);
    }
    let completion = VerifiedRailCompletion {
        audience: reader.text_u16()?,
        source: reader.text_u16()?,
        key_epoch: reader.u64()?,
        message_id: reader.array()?,
        signed_meaning_digest: Sha256::digest(signed).into(),
        issued_at_seconds: reader.u64()?,
        expires_at_seconds: reader.u64()?,
        protocol_identity: reader.text_u16()?,
        protocol_version: reader.u16()?,
        correlation_family: reader.text_u16()?,
        correlation_token: reader.bytes_u16()?.to_vec(),
        payload: reader.bytes_u32()?.to_vec(),
    };
    if !reader.remaining().is_empty() {
        return Err(CompletionWireDenial::Malformed);
    }
    if completion.audience != expected.audience
        || completion.source != expected.source
        || completion.key_epoch != expected.key_epoch
    {
        return Err(CompletionWireDenial::WrongSource);
    }
    if completion.expires_at_seconds < completion.issued_at_seconds
        || completion.issued_at_seconds
            > expected
                .now_seconds
                .saturating_add(expected.maximum_clock_skew_seconds)
        || completion.expires_at_seconds < expected.now_seconds
    {
        return Err(CompletionWireDenial::Expired);
    }
    Ok(completion)
}

struct Reader<'a> {
    remaining: &'a [u8],
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { remaining: bytes }
    }

    fn remaining(&self) -> &'a [u8] {
        self.remaining
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], CompletionWireDenial> {
        let (head, tail) = self
            .remaining
            .split_at_checked(count)
            .ok_or(CompletionWireDenial::Malformed)?;
        self.remaining = tail;
        Ok(head)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], CompletionWireDenial> {
        self.take(N)?
            .try_into()
            .map_err(|_| CompletionWireDenial::Malformed)
    }

    fn u16(&mut self) -> Result<u16, CompletionWireDenial> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, CompletionWireDenial> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, CompletionWireDenial> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn bytes_u16(&mut self) -> Result<&'a [u8], CompletionWireDenial> {
        let count = usize::from(self.u16()?);
        self.take(count)
    }

    fn bytes_u32(&mut self) -> Result<&'a [u8], CompletionWireDenial> {
        let count = usize::try_from(self.u32()?).map_err(|_| CompletionWireDenial::Malformed)?;
        self.take(count)
    }

    fn text_u16(&mut self) -> Result<String, CompletionWireDenial> {
        std::str::from_utf8(self.bytes_u16()?)
            .map(str::to_owned)
            .map_err(|_| CompletionWireDenial::Malformed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixed bytes emitted by the independent rail encoder with seed [7; 32].
    const RAIL_V1_HEX: &str = include_str!("inbound_completion_v1.hex");
    const RAIL_PUBLIC_KEY_HEX: &str =
        "ea4a6c63e29c520abef5507b132ec5f9954776aebebe7b92421eea691446d22c";

    #[test]
    fn fixed_rail_bytes_verify_and_boundary_substitutions_deny() {
        let body = decode_hex(RAIL_V1_HEX.trim());
        assert_eq!(body.len(), 292);
        let key =
            VerifyingKey::from_bytes(&decode_hex(RAIL_PUBLIC_KEY_HEX).try_into().unwrap()).unwrap();
        let expected = || RailCompletionVerification {
            key: &key,
            audience: "bank-process-court",
            source: "rail-primary",
            key_epoch: 1,
            now_seconds: 1_700_000_000,
            maximum_clock_skew_seconds: 0,
        };
        let completion = verify_completion(&body, expected()).unwrap();
        assert_eq!(completion.message_id.len(), 32);
        assert_eq!(
            completion.protocol_identity,
            "bank.estate.death-notification"
        );
        assert_eq!(completion.protocol_version, 1);
        assert_eq!(completion.correlation_family, "estate-death-notice-rail");
        assert_eq!(completion.correlation_token, [4; 32]);
        assert_eq!(completion.payload.len(), 24);
        assert_eq!(completion.expires_at_seconds, 1_700_000_060);

        let mut tampered = body.clone();
        tampered[130] ^= 1;
        assert_eq!(
            verify_completion(&tampered, expected()),
            Err(CompletionWireDenial::BadSignature)
        );
        assert_eq!(
            verify_completion(
                &body,
                RailCompletionVerification {
                    audience: "foreign-bank",
                    ..expected()
                }
            ),
            Err(CompletionWireDenial::WrongSource)
        );
        assert_eq!(
            verify_completion(
                &body,
                RailCompletionVerification {
                    now_seconds: 1_700_000_061,
                    ..expected()
                }
            ),
            Err(CompletionWireDenial::Expired)
        );
        assert_eq!(
            verify_completion(&vec![0; MAXIMUM_COMPLETION_BYTES + 1], expected()),
            Err(CompletionWireDenial::Oversized)
        );
    }

    fn decode_hex(hex: &str) -> Vec<u8> {
        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
}
