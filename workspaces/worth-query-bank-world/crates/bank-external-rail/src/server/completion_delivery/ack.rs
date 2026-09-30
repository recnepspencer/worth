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

#[cfg(test)]
mod tests {
    use ed25519_dalek::SigningKey;
    use sha2::{Digest, Sha256};

    use super::*;

    #[test]
    fn fixed_bank_ack_verifies_exact_completion() {
        let body = include_str!("estate_v1.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        let message_id: [u8; 32] = body[58..90].try_into().unwrap();
        let digest: [u8; 32] = Sha256::digest(&body).into();
        let completion = SignedRailCompletion::new(body, message_id, digest);
        let ack = include_str!("custody_ack_v1.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            verify_ack(
                &ack,
                &completion,
                &SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes()
            ),
            Some(RailCustodyAckPosture::AcceptedPending)
        );
        let mut altered = ack;
        altered[49] ^= 1;
        assert_eq!(
            verify_ack(
                &altered,
                &completion,
                &SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes()
            ),
            None
        );
    }
}
