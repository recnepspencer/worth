//! Independent host expectations compared with descriptive envelope claims.

use worth_query_installation::facade::WorthQueryPortableDomainPackageIdentity;
use worth_query_package_archive::facade::WorthQueryUnsignedPackageReleaseEnvelope;

use crate::denial::WorthQueryReleaseCeremonyError as Error;

pub(crate) struct ExpectedRelease {
    package_identity: WorthQueryPortableDomainPackageIdentity,
    name: String,
    version: String,
}

impl ExpectedRelease {
    pub(crate) fn parse(
        package_identity: &str,
        name: String,
        version: String,
    ) -> Result<Self, Error> {
        Ok(Self {
            package_identity: parse_package_identity(package_identity)?,
            name,
            version,
        })
    }
}

pub(crate) struct ExpectedProvenance {
    repository: String,
    revision: String,
    reference: String,
}

impl ExpectedProvenance {
    pub(crate) const fn new(repository: String, revision: String, reference: String) -> Self {
        Self {
            repository,
            revision,
            reference,
        }
    }
}

pub(crate) struct ExpectedSigner {
    identity: String,
    protocol_identity: String,
    protocol_version: u32,
    signature_bytes: u32,
}

impl ExpectedSigner {
    pub(crate) const fn new(
        identity: String,
        protocol_identity: String,
        protocol_version: u32,
        signature_bytes: u32,
    ) -> Self {
        Self {
            identity,
            protocol_identity,
            protocol_version,
            signature_bytes,
        }
    }
}

pub(crate) struct ReleaseExpectations {
    release: ExpectedRelease,
    provenance: ExpectedProvenance,
    signer: ExpectedSigner,
}

impl ReleaseExpectations {
    pub(crate) const fn new(
        release: ExpectedRelease,
        provenance: ExpectedProvenance,
        signer: ExpectedSigner,
    ) -> Self {
        Self {
            release,
            provenance,
            signer,
        }
    }

    pub(crate) const fn package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        &self.release.package_identity
    }

    pub(crate) const fn signature_bytes(&self) -> u32 {
        self.signer.signature_bytes
    }

    pub(crate) fn admit_signature_bytes(&self, observed: usize) -> Result<(), Error> {
        let observed = u32::try_from(observed).map_err(|_| Error::ExpectationMismatch {
            field: "signature byte count",
        })?;
        require_equal(
            observed,
            self.signer.signature_bytes,
            "signature byte count",
        )
    }

    pub(crate) fn admit(
        &self,
        envelope: &WorthQueryUnsignedPackageReleaseEnvelope,
    ) -> Result<(), Error> {
        require_equal(
            envelope.expected_package_identity(),
            &self.release.package_identity,
            "package identity claim",
        )?;
        let release = envelope.release_metadata();
        require_equal(
            release.release_name(),
            self.release.name.as_str(),
            "release name",
        )?;
        require_equal(
            release.release_version(),
            self.release.version.as_str(),
            "release version",
        )?;
        admit_provenance(envelope, &self.provenance)?;
        admit_signer(envelope, &self.signer)
    }
}

fn admit_provenance(
    unsigned: &worth_query_package_archive::facade::WorthQueryUnsignedPackageReleaseEnvelope,
    expected: &ExpectedProvenance,
) -> Result<(), Error> {
    let observed = unsigned.provenance();
    require_equal(
        observed.source_repository(),
        expected.repository.as_str(),
        "source repository",
    )?;
    require_equal(
        observed.source_revision(),
        expected.revision.as_str(),
        "source revision",
    )?;
    require_equal(
        observed.source_reference(),
        expected.reference.as_str(),
        "source reference",
    )
}

fn admit_signer(
    unsigned: &worth_query_package_archive::facade::WorthQueryUnsignedPackageReleaseEnvelope,
    expected: &ExpectedSigner,
) -> Result<(), Error> {
    let observed = unsigned.signer();
    require_equal(
        observed.signer_identity(),
        expected.identity.as_str(),
        "signer identity",
    )?;
    require_equal(
        observed.signature_protocol_identity().as_str(),
        expected.protocol_identity.as_str(),
        "signature protocol identity",
    )?;
    require_equal(
        observed.signature_protocol_version().get(),
        expected.protocol_version,
        "signature protocol version",
    )
}

fn require_equal<T: Eq>(observed: T, expected: T, field: &'static str) -> Result<(), Error> {
    if observed == expected {
        Ok(())
    } else {
        Err(Error::ExpectationMismatch { field })
    }
}

fn parse_package_identity(value: &str) -> Result<WorthQueryPortableDomainPackageIdentity, Error> {
    if value.len() != 64 {
        return Err(Error::InvalidPackageIdentity);
    }
    let mut bytes = [0_u8; 32];
    for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        bytes[index] = decode_hex_byte(pair).ok_or(Error::InvalidPackageIdentity)?;
    }
    Ok(WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(bytes))
}

fn decode_hex_byte(pair: &[u8]) -> Option<u8> {
    let high = decode_hex_digit(pair[0])?;
    let low = decode_hex_digit(pair[1])?;
    Some((high << 4) | low)
}

const fn decode_hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}
