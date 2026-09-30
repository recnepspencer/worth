use super::WorthQueryInboundOccurrenceClaims;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

/// Installed product mechanism for authenticating one bounded envelope.
/// Implementations must verify exact bytes, source key, audience and validity
/// against the supplied Query clock sample before returning claims. Query checks those claims against its
/// installed operation and authoritative dispatch provenance.
/// The trusted implementation must conservatively account its own bounded
/// parsing and authentication work against `maximum_work` and return
/// `WorkExhausted` before exceeding it. Work units are mechanism-defined; the
/// numeric limit does not preempt arbitrary product code.
/// `signed_meaning_digest` must cover every authenticated semantic field of
/// the signed body, including audience, source, key epoch, message identity,
/// validity times, protocol, correlation and payload. It must exclude only
/// detached signature material; changing any signed field changes the digest.
pub trait WorthQueryInboundOccurrenceVerifier: Send + Sync {
    fn audience(&self) -> &str;
    fn source_identity(&self) -> &str;
    fn protocol_identity(&self) -> &BoundaryProtocolIdentity;
    fn protocol_version(&self) -> BoundaryProtocolVersion;

    fn verify(
        &self,
        envelope: &[u8],
        now_unix_seconds: u64,
        maximum_work: std::num::NonZeroU64,
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
    WorkExhausted,
    Expired,
}
