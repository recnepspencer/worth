use std::num::NonZeroU64;

use ed25519_dalek::VerifyingKey;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundOccurrenceClaims, WorthQueryInboundOccurrenceVerifier,
    WorthQueryInboundVerificationDenial,
};

use crate::http::protocol::inbound_completion::{
    verify_completion, CompletionWireDenial, RailCompletionVerification,
};

/// Bank's installed rail source mechanism. The HTTP route supplies bytes;
/// this verifier independently authenticates them before Query sees claims.
pub(in crate::http::server) struct BankRailCompletionVerifier {
    key: VerifyingKey,
    audience: String,
    source: String,
    key_epoch: u64,
    maximum_clock_skew_seconds: u64,
    protocol_identity: BoundaryProtocolIdentity,
    protocol_version: BoundaryProtocolVersion,
}

impl BankRailCompletionVerifier {
    pub(in crate::http::server) fn new(
        key: [u8; 32],
        audience: String,
        source: String,
        key_epoch: u64,
        maximum_clock_skew_seconds: u64,
        protocol_identity: BoundaryProtocolIdentity,
        protocol_version: BoundaryProtocolVersion,
    ) -> Option<Self> {
        if audience.is_empty() || source.is_empty() || key_epoch == 0 {
            return None;
        }
        Some(Self {
            key: VerifyingKey::from_bytes(&key).ok()?,
            audience,
            source,
            key_epoch,
            maximum_clock_skew_seconds,
            protocol_identity,
            protocol_version,
        })
    }
}

impl WorthQueryInboundOccurrenceVerifier for BankRailCompletionVerifier {
    fn audience(&self) -> &str {
        &self.audience
    }

    fn source_identity(&self) -> &str {
        &self.source
    }

    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.protocol_identity
    }

    fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.protocol_version
    }

    fn verify(
        &self,
        envelope: &[u8],
        now_seconds: u64,
        maximum_work: NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        // Four envelope-length byte traversals cover signature input, decode,
        // text validation and digest, plus one fixed Ed25519 operation.
        // Refuse before the cryptographic or parsing work begins.
        let byte_work = u64::try_from(envelope.len())
            .ok()
            .and_then(|length| length.checked_mul(4))
            .and_then(|work| work.checked_add(64))
            .ok_or(WorthQueryInboundVerificationDenial::WorkExhausted)?;
        if byte_work > maximum_work.get() {
            return Err(WorthQueryInboundVerificationDenial::WorkExhausted);
        }
        let verified = verify_completion(
            envelope,
            RailCompletionVerification {
                key: &self.key,
                audience: &self.audience,
                source: &self.source,
                key_epoch: self.key_epoch,
                now_seconds,
                maximum_clock_skew_seconds: self.maximum_clock_skew_seconds,
            },
        )
        .map_err(map_wire_denial)?;
        if u32::from(verified.protocol_version) != self.protocol_version.get() {
            return Err(WorthQueryInboundVerificationDenial::UnsupportedVersion);
        }
        Ok(WorthQueryInboundOccurrenceClaims {
            audience: verified.audience,
            source_identity: verified.source,
            key_epoch: verified.key_epoch,
            message_identity: verified.message_id,
            signed_meaning_digest: verified.signed_meaning_digest,
            issued_at_unix_seconds: verified.issued_at_seconds,
            expires_at_unix_seconds: verified.expires_at_seconds,
            protocol_identity: worth_foundational::facade::BoundaryProtocolIdentity::parse(
                verified.protocol_identity,
            )
            .map_err(|_| WorthQueryInboundVerificationDenial::Malformed)?,
            protocol_version: worth_foundational::facade::BoundaryProtocolVersion::try_new(
                u32::from(verified.protocol_version),
            )
            .map_err(|_| WorthQueryInboundVerificationDenial::UnsupportedVersion)?,
            correlation_family: verified.correlation_family,
            correlation_token: verified
                .correlation_token
                .try_into()
                .map_err(|_| WorthQueryInboundVerificationDenial::Malformed)?,
            payload: verified.payload,
        })
    }
}

fn map_wire_denial(denial: CompletionWireDenial) -> WorthQueryInboundVerificationDenial {
    match denial {
        CompletionWireDenial::Oversized => WorthQueryInboundVerificationDenial::Oversized,
        CompletionWireDenial::Malformed => WorthQueryInboundVerificationDenial::Malformed,
        CompletionWireDenial::BadSignature | CompletionWireDenial::WrongSource => {
            WorthQueryInboundVerificationDenial::AuthenticationFailed
        }
        CompletionWireDenial::Expired => WorthQueryInboundVerificationDenial::Expired,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_refuses_insufficient_work_before_decoding_signed_bytes() {
        let verifier = BankRailCompletionVerifier::new(
            ed25519_dalek::SigningKey::from_bytes(&[7; 32])
                .verifying_key()
                .to_bytes(),
            "bank-process-court".into(),
            "rail-primary".into(),
            1,
            0,
            BoundaryProtocolIdentity::new("bank.estate.death-notification"),
            BoundaryProtocolVersion::new(1),
        )
        .unwrap();
        let envelope = include_str!("../../protocol/inbound_completion_v1.hex")
            .trim()
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect::<Vec<_>>();
        assert!(matches!(
            verifier.verify(&envelope, 1_700_000_000, NonZeroU64::new(1).unwrap()),
            Err(WorthQueryInboundVerificationDenial::WorkExhausted)
        ));
        assert!(verifier
            .verify(&envelope, 1_700_000_000, NonZeroU64::new(20_480).unwrap())
            .is_ok());
    }
}
