use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

/// Decoded claims from a product verifier. This value alone cannot accept or
/// complete an occurrence; Query checks the installed binding and owner row.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInboundOccurrenceClaims {
    pub audience: String,
    pub source_identity: String,
    pub key_epoch: u64,
    pub message_identity: [u8; 32],
    /// Digest of the authenticated signed body, excluding its detached signature.
    pub signed_meaning_digest: [u8; 32],
    pub issued_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub protocol_identity: BoundaryProtocolIdentity,
    pub protocol_version: BoundaryProtocolVersion,
    pub correlation_family: String,
    pub correlation_token: [u8; 32],
    pub payload: Vec<u8>,
}
