//! Query-issued proof of a definitive owner conflict after authentication.

use sha2::{Digest, Sha256};

/// An authenticated conflict whose exact owner meaning cannot be retried as new.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundPermanentDenialKind {
    MessageIdentityConflict,
    CorrelationAlreadyOwned,
}

/// Sealed evidence for a product signer. The verifier authenticated the full
/// envelope before Query decided the exact owner conflict. Neither HTTP status
/// nor caller-supplied bytes can construct this result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryInboundAuthenticatedPermanentDenial {
    kind: WorthQueryInboundPermanentDenialKind,
    message_identity: [u8; 32],
    envelope_digest: [u8; 32],
}

impl WorthQueryInboundAuthenticatedPermanentDenial {
    pub const fn kind(&self) -> WorthQueryInboundPermanentDenialKind {
        self.kind
    }

    pub const fn message_identity(&self) -> &[u8; 32] {
        &self.message_identity
    }

    pub const fn envelope_digest(&self) -> &[u8; 32] {
        &self.envelope_digest
    }

    pub(super) fn seal(
        kind: WorthQueryInboundPermanentDenialKind,
        message_identity: [u8; 32],
        envelope: &[u8],
    ) -> Self {
        Self {
            kind,
            message_identity,
            envelope_digest: Sha256::digest(envelope).into(),
        }
    }
}
