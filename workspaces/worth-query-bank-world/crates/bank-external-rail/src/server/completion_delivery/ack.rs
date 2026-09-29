//! Strict verification of Bank's exact custody response.

use ed25519_dalek::{Signature, VerifyingKey};

use crate::completion_wire::{
    RailCustodyAckPosture, SignedRailCompletion, CUSTODY_ACK_V1_BYTES, CUSTODY_ACK_V1_MAGIC,
    SIGNATURE_BYTES,
};

pub(super) fn verify_ack(
    bytes: &[u8],
    completion: &SignedRailCompletion,
    key: &[u8; 32],
) -> Option<RailCustodyAckPosture> {
    if bytes.len() != CUSTODY_ACK_V1_BYTES {
        return None;
    }
    let signed_length = bytes.len() - SIGNATURE_BYTES;
    let signed = &bytes[..signed_length];
    if &signed[..16] != CUSTODY_ACK_V1_MAGIC
        || &signed[17..49] != completion.message_id()
        || &signed[49..81] != completion.digest()
    {
        return None;
    }
    let posture = RailCustodyAckPosture::from_byte(signed[16])?;
    let signature_bytes: [u8; SIGNATURE_BYTES] = bytes[signed_length..].try_into().ok()?;
    let signature = Signature::from_bytes(&signature_bytes);
    VerifyingKey::from_bytes(key)
        .ok()?
        .verify_strict(signed, &signature)
        .ok()?;
    Some(posture)
}
