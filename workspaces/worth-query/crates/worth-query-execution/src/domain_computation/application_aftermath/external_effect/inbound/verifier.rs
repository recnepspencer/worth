use super::WorthQueryInboundOccurrenceClaims;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

/// Installed product mechanism for authenticating one bounded envelope.
/// Implementations must verify exact bytes, source key, audience and validity
/// against the supplied Query clock sample before returning claims. Query checks those claims against its
/// installed operation and authoritative dispatch provenance.
pub trait WorthQueryInboundOccurrenceVerifier: Send + Sync {
    fn audience(&self) -> &str;
    fn source_identity(&self) -> &str;
    fn protocol_identity(&self) -> &BoundaryProtocolIdentity;
    fn protocol_version(&self) -> BoundaryProtocolVersion;

    fn verify(
        &self,
        envelope: &[u8],
        now_unix_seconds: u64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial>;
}

/// The installed source verifier rejected the envelope before owner admission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInboundVerificationDenial {
    AuthenticationFailed,
    UnsupportedVersion,
    SourceUnavailable,
    TimeUnavailable,
    Malformed,
    Oversized,
    Expired,
}
