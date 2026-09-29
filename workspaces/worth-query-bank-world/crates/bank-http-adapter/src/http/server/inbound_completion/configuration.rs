//! Trusted process installation for one Bank rail completion source.

use ed25519_dalek::{SigningKey, VerifyingKey};
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub struct BankRailCompletionServerInstallation {
    pub(super) rail_verifying_key: [u8; 32],
    pub(super) bank_ack_signing_seed: [u8; 32],
    pub(super) audience: String,
    pub(super) source: String,
    pub(super) key_epoch: u64,
    pub(super) maximum_clock_skew_seconds: u64,
}

impl BankRailCompletionServerInstallation {
    pub fn new(
        rail_verifying_key: [u8; 32],
        bank_ack_signing_seed: [u8; 32],
        audience: String,
        source: String,
        key_epoch: u64,
        maximum_clock_skew_seconds: u64,
    ) -> Option<Self> {
        VerifyingKey::from_bytes(&rail_verifying_key).ok()?;
        if audience.is_empty() || source != "rail-primary" || key_epoch == 0 {
            return None;
        }
        Some(Self {
            rail_verifying_key,
            bank_ack_signing_seed,
            audience,
            source,
            key_epoch,
            maximum_clock_skew_seconds,
        })
    }

    pub(super) fn verifier(&self) -> super::BankRailCompletionVerifier {
        super::BankRailCompletionVerifier::new(
            self.rail_verifying_key,
            self.audience.clone(),
            self.source.clone(),
            self.key_epoch,
            self.maximum_clock_skew_seconds,
            BoundaryProtocolIdentity::new("bank.estate.death-notification"),
            BoundaryProtocolVersion::new(1),
        )
        .expect("validated rail completion installation")
    }

    pub(super) fn ack_signer(&self) -> SigningKey {
        SigningKey::from_bytes(&self.bank_ack_signing_seed)
    }
}
