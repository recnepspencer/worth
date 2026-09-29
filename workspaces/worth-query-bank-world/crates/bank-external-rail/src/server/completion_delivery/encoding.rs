//! Rail-owned encoder. Bank independently decodes the documented wire format.

use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};

use crate::completion_wire::{SignedRailCompletion, COMPLETION_V1_MAGIC, MAXIMUM_ENVELOPE_BYTES};
use crate::protocol::correlation::RailCorrelation;
use crate::protocol::payload::RailEffectPayload;

use super::RailCompletionDeliveryConfiguration;

pub(super) fn sign_completed_effect(
    configuration: &RailCompletionDeliveryConfiguration,
    correlation: &RailCorrelation,
    payload: &RailEffectPayload,
    issued: u64,
) -> Result<SignedRailCompletion, CompletionEncodingDenial> {
    let expiry = issued
        .checked_add(configuration.validity_seconds)
        .ok_or(CompletionEncodingDenial::InvalidTime)?;
    let mut identity_basis = Vec::new();
    identity_basis.extend_from_slice(b"bank-rail-completion-id-v1");
    put_text(&mut identity_basis, &configuration.audience)?;
    put_text(&mut identity_basis, &configuration.source)?;
    identity_basis.extend_from_slice(&configuration.key_epoch.to_be_bytes());
    put_text(&mut identity_basis, payload.protocol_identity().as_str())?;
    let version = u16::try_from(payload.protocol_version().get())
        .map_err(|_| CompletionEncodingDenial::OversizedField)?;
    identity_basis.extend_from_slice(&version.to_be_bytes());
    put_text(&mut identity_basis, correlation.family())?;
    put_short_bytes(&mut identity_basis, correlation.token())?;
    put_long_bytes(&mut identity_basis, payload.bytes())?;
    let message_id: [u8; 32] = Sha256::digest(identity_basis).into();

    let mut signed = Vec::with_capacity(MAXIMUM_ENVELOPE_BYTES.min(256 + payload.bytes().len()));
    signed.extend_from_slice(COMPLETION_V1_MAGIC);
    put_text(&mut signed, &configuration.audience)?;
    put_text(&mut signed, &configuration.source)?;
    signed.extend_from_slice(&configuration.key_epoch.to_be_bytes());
    signed.extend_from_slice(&message_id);
    signed.extend_from_slice(&issued.to_be_bytes());
    signed.extend_from_slice(&expiry.to_be_bytes());
    put_text(&mut signed, payload.protocol_identity().as_str())?;
    signed.extend_from_slice(&version.to_be_bytes());
    put_text(&mut signed, correlation.family())?;
    put_short_bytes(&mut signed, correlation.token())?;
    put_long_bytes(&mut signed, payload.bytes())?;
    if signed.len() + 64 > MAXIMUM_ENVELOPE_BYTES {
        return Err(CompletionEncodingDenial::OversizedEnvelope);
    }
    let signature = SigningKey::from_bytes(&configuration.signing_seed).sign(&signed);
    signed.extend_from_slice(&signature.to_bytes());
    let digest: [u8; 32] = Sha256::digest(&signed).into();
    Ok(SignedRailCompletion::new(signed, message_id, digest))
}

fn put_text(buffer: &mut Vec<u8>, value: &str) -> Result<(), CompletionEncodingDenial> {
    put_short_bytes(buffer, value.as_bytes())
}

fn put_short_bytes(buffer: &mut Vec<u8>, value: &[u8]) -> Result<(), CompletionEncodingDenial> {
    let length =
        u16::try_from(value.len()).map_err(|_| CompletionEncodingDenial::OversizedField)?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(value);
    Ok(())
}

fn put_long_bytes(buffer: &mut Vec<u8>, value: &[u8]) -> Result<(), CompletionEncodingDenial> {
    let length =
        u32::try_from(value.len()).map_err(|_| CompletionEncodingDenial::OversizedField)?;
    buffer.extend_from_slice(&length.to_be_bytes());
    buffer.extend_from_slice(value);
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionEncodingDenial {
    InvalidTime,
    OversizedField,
    OversizedEnvelope,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use ed25519_dalek::SigningKey;
    use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

    use super::*;

    #[test]
    fn deterministic_v1_fixture() {
        let configuration = RailCompletionDeliveryConfiguration::new(
            "http://127.0.0.1:1/v1/inbound/rail-completions".into(),
            "bank-process-court".into(),
            "rail-primary".into(),
            1,
            [7; 32],
            SigningKey::from_bytes(&[9; 32]).verifying_key().to_bytes(),
            60,
            2,
            3,
            Duration::from_millis(20),
            Duration::from_secs(1),
            None,
        )
        .unwrap();
        let payload = [3u64, 12, 1]
            .into_iter()
            .flat_map(u64::to_be_bytes)
            .collect::<Vec<_>>();
        let signed = sign_completed_effect(
            &configuration,
            &RailCorrelation::new("estate-death-notice-rail", [4u8; 32]),
            &RailEffectPayload::new(
                "EstateDeathNotificationEffect",
                BoundaryProtocolIdentity::new("bank.estate.death-notification"),
                BoundaryProtocolVersion::new(1),
                24,
                payload,
            ),
            1_700_000_000,
        )
        .unwrap();
        println!(
            "FIXTURE_BODY={}",
            signed
                .bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        println!(
            "FIXTURE_PUBLIC_KEY={}",
            SigningKey::from_bytes(&[7; 32])
                .verifying_key()
                .to_bytes()
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        );
        assert_eq!(signed.bytes().len(), 292);
    }
}
