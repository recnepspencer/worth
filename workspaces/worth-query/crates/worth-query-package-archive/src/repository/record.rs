//! Bounded descriptive records crossing the repository boundary.

use worth_query_declaration::archive_decode_authority::ArchiveDecodeAuthority;
use worth_query_installation::facade::WorthQueryPortableDomainPackageIdentity;

use crate::denial::WorthQueryPackageArchiveDenial;
use crate::envelope::{
    decode_package_release_envelope, encode_package_release_envelope,
    WorthQueryPackageEnvelopeLimits, WorthQuerySignedPackageReleaseEnvelope,
};

use super::{
    WorthQueryExactPackageArchiveRequest, WorthQueryPackageArchiveRepositoryDenial,
    WorthQueryPackageArchiveRepositoryDenialKind,
};

/// Immutable canonical envelope bytes keyed by their embedded claimed identity.
///
/// Signature presence is descriptive. This record carries no signer trust,
/// Query validation, repository activation, or runtime authority.
#[derive(Debug, Eq, PartialEq)]
pub struct WorthQuerySignedPackageArchiveRecord {
    claimed_package_identity: WorthQueryPortableDomainPackageIdentity,
    envelope_bytes: Vec<u8>,
}

impl WorthQuerySignedPackageArchiveRecord {
    pub fn from_signed_envelope(
        envelope: WorthQuerySignedPackageReleaseEnvelope,
        limits: WorthQueryPackageEnvelopeLimits,
    ) -> Result<Self, WorthQueryPackageArchiveDenial> {
        let claimed_package_identity = envelope.unsigned().expected_package_identity().clone();
        let envelope_bytes = encode_package_release_envelope(&envelope, limits)?;
        Ok(Self {
            claimed_package_identity,
            envelope_bytes,
        })
    }

    pub fn from_untrusted_envelope_bytes(
        _: ArchiveDecodeAuthority,
        envelope_bytes: Vec<u8>,
        limits: WorthQueryPackageEnvelopeLimits,
    ) -> Result<Self, WorthQueryPackageArchiveDenial> {
        let decoded = decode_package_release_envelope(&envelope_bytes, limits)?;
        let claimed_package_identity = decoded.expected_package_identity().clone();
        Ok(Self {
            claimed_package_identity,
            envelope_bytes,
        })
    }

    pub const fn claimed_package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        &self.claimed_package_identity
    }

    pub fn exact_envelope_bytes(&self) -> &[u8] {
        &self.envelope_bytes
    }

    pub fn into_exact_envelope_bytes(self) -> Vec<u8> {
        self.envelope_bytes
    }
}

/// Bounded bytes returned for one independently expected identity.
///
/// The requested identity records lookup intent only. The returned envelope's
/// claimed identity and meaning remain untrusted until decode, host-policy
/// verification, reconstruction, and fresh Query validation all succeed.
#[derive(Debug, Eq, PartialEq)]
pub struct WorthQueryUntrustedLoadedPackageArchive {
    request: WorthQueryExactPackageArchiveRequest,
    envelope_bytes: Vec<u8>,
}

impl WorthQueryUntrustedLoadedPackageArchive {
    pub fn from_untrusted_bytes(
        _: ArchiveDecodeAuthority,
        request: WorthQueryExactPackageArchiveRequest,
        envelope_bytes: Vec<u8>,
    ) -> Result<Self, WorthQueryPackageArchiveRepositoryDenial> {
        let observed_bytes = u64::try_from(envelope_bytes.len()).unwrap_or(u64::MAX);
        if observed_bytes > request.envelope_limits().maximum_envelope_bytes() {
            return Err(WorthQueryPackageArchiveRepositoryDenial::new(
                WorthQueryPackageArchiveRepositoryDenialKind::EnvelopeByteBudgetExceeded,
            ));
        }
        Ok(Self {
            request,
            envelope_bytes,
        })
    }

    pub const fn requested_package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        self.request.expected_package_identity()
    }

    pub const fn envelope_limits(&self) -> WorthQueryPackageEnvelopeLimits {
        self.request.envelope_limits()
    }

    pub fn untrusted_envelope_bytes(&self) -> &[u8] {
        &self.envelope_bytes
    }

    pub fn into_untrusted_envelope_bytes(self) -> Vec<u8> {
        self.envelope_bytes
    }
}
