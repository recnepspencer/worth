//! Canonical re-entry for untrusted release-envelope signing payloads.

use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryExecutionAccessProductFamily, WorthQueryExecutionAllocatorFamily,
    WorthQueryExecutionProviderFamily, WorthQueryExecutionProviderRequirements,
    WorthQueryInstallationCapabilityFamily, WorthQueryInstallationConfigSectionFamily,
    WorthQueryInstallationOperatingRequirement, WorthQueryPortableDomainPackageIdentity,
};

use crate::binary_input::BinaryInput;
use crate::compatibility::{
    WorthQueryPackageArchiveCompatibilityProfile, WorthQueryPackageArchiveProtocolLayer,
};
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

use super::{
    encode_signing_payload, validate_descriptive_text, WorthQueryPackageArchiveChecksum,
    WorthQueryPackageBuildMetadata, WorthQueryPackageEnvelopeLimits,
    WorthQueryPackageReleaseEnvelopeDescriptor, WorthQueryPackageReleaseMetadata,
    WorthQueryPackageReleaseProvenance, WorthQueryPackageReleaseRequirements,
    WorthQueryPackageReleaseSignerDescriptor, WorthQueryUnsignedPackageReleaseEnvelope,
    ENVELOPE_MAGIC,
};

/// Canonical unsigned release description that has not passed host trust policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUntrustedPackageReleaseSigningPayload {
    unsigned: WorthQueryUnsignedPackageReleaseEnvelope,
}

impl WorthQueryUntrustedPackageReleaseSigningPayload {
    pub(crate) const fn new(unsigned: WorthQueryUnsignedPackageReleaseEnvelope) -> Self {
        Self { unsigned }
    }

    pub const fn unsigned(&self) -> &WorthQueryUnsignedPackageReleaseEnvelope {
        &self.unsigned
    }

    pub fn signing_payload(&self) -> &[u8] {
        self.unsigned.signing_payload()
    }

    pub fn archive(&self) -> &[u8] {
        self.unsigned.archive()
    }

    pub const fn expected_package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        self.unsigned.expected_package_identity()
    }

    pub fn into_unsigned(self) -> WorthQueryUnsignedPackageReleaseEnvelope {
        self.unsigned
    }
}

/// Decode one complete canonical signing payload without attaching trust.
pub fn decode_package_release_signing_payload(
    bytes: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
) -> Result<WorthQueryUntrustedPackageReleaseSigningPayload, Denial> {
    let (payload, consumed) = decode_package_release_signing_payload_prefix(bytes, limits)?;
    if consumed != bytes.len() {
        return Err(Denial::new(Kind::TrailingBytes));
    }
    Ok(payload)
}

pub(super) fn decode_package_release_signing_payload_prefix(
    bytes: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
) -> Result<(WorthQueryUntrustedPackageReleaseSigningPayload, usize), Denial> {
    let limits = limits.narrowed();
    require_envelope_budget(bytes.len(), limits)?;
    let mut input = BinaryInput::new(bytes);
    require_header(&mut input)?;
    let body_bytes = input.u64()?;
    let body_length =
        usize::try_from(body_bytes).map_err(|_| Denial::new(Kind::InvalidEnvelopeLength))?;
    let body = EnvelopeBodyDecoder::new(input.take(body_length)?, limits).decode()?;
    let consumed = bytes
        .len()
        .checked_sub(input.remaining_len())
        .ok_or_else(|| Denial::new(Kind::InvalidEnvelopeLength))?;
    let signing_payload = bytes
        .get(..consumed)
        .ok_or_else(|| Denial::new(Kind::InvalidEnvelopeLength))?;
    require_checksum(body.archive_checksum, &body.archive)?;
    require_canonical_signing_payload(&body, signing_payload, limits)?;
    let unsigned = WorthQueryUnsignedPackageReleaseEnvelope::new(
        body.archive,
        body.expected_package_identity,
        body.archive_checksum,
        body.descriptor,
        body.requirements,
        signing_payload.to_vec(),
    );
    Ok((
        WorthQueryUntrustedPackageReleaseSigningPayload::new(unsigned),
        consumed,
    ))
}

pub(super) fn require_envelope_budget(
    observed: usize,
    limits: WorthQueryPackageEnvelopeLimits,
) -> Result<(), Denial> {
    if u64::try_from(observed).unwrap_or(u64::MAX) > limits.maximum_envelope_bytes() {
        return Err(Denial::new(Kind::EnvelopeByteBudgetExceeded));
    }
    Ok(())
}

fn require_header(input: &mut BinaryInput<'_>) -> Result<(), Denial> {
    if input.array::<8>()? != ENVELOPE_MAGIC {
        return Err(Denial::new(Kind::InvalidMagic));
    }
    WorthQueryPackageArchiveCompatibilityProfile::CURRENT
        .admit(
            WorthQueryPackageArchiveProtocolLayer::ReleaseEnvelope,
            input.u16()?,
        )
        .map_err(|compatibility| {
            Denial::incompatible(Kind::UnsupportedEnvelopeVersion, compatibility)
        })
}

fn require_checksum(
    checksum: WorthQueryPackageArchiveChecksum,
    archive: &[u8],
) -> Result<(), Denial> {
    if !checksum.matches(archive) {
        return Err(Denial::new(Kind::ArchiveChecksumMismatch));
    }
    Ok(())
}

fn require_canonical_signing_payload(
    body: &DecodedEnvelopeBody,
    observed: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
) -> Result<(), Denial> {
    let canonical = encode_signing_payload(
        &body.archive,
        &body.expected_package_identity,
        body.archive_checksum,
        &body.requirements,
        &body.descriptor,
        limits,
    )?;
    if canonical != observed {
        return Err(Denial::new(Kind::InvalidEnvelopeLength));
    }
    Ok(())
}

struct DecodedEnvelopeBody {
    archive: Vec<u8>,
    expected_package_identity: WorthQueryPortableDomainPackageIdentity,
    archive_checksum: WorthQueryPackageArchiveChecksum,
    descriptor: WorthQueryPackageReleaseEnvelopeDescriptor,
    requirements: WorthQueryPackageReleaseRequirements,
}

struct EnvelopeBodyDecoder<'a> {
    input: BinaryInput<'a>,
    limits: WorthQueryPackageEnvelopeLimits,
    descriptive_text_bytes: u64,
    requirements: u32,
}

impl<'a> EnvelopeBodyDecoder<'a> {
    const fn new(bytes: &'a [u8], limits: WorthQueryPackageEnvelopeLimits) -> Self {
        Self {
            input: BinaryInput::new(bytes),
            limits,
            descriptive_text_bytes: 0,
            requirements: 0,
        }
    }

    fn decode(mut self) -> Result<DecodedEnvelopeBody, Denial> {
        let expected_package_identity =
            WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(
                ARCHIVE_DECODE,
                self.input.array()?,
            );
        let archive_checksum =
            WorthQueryPackageArchiveChecksum::from_untrusted_bytes(self.input.array()?);
        let build_metadata = WorthQueryPackageBuildMetadata::new(
            self.text()?,
            self.text()?,
            self.text()?,
            self.text()?,
            self.text()?,
        )?;
        let release_metadata = WorthQueryPackageReleaseMetadata::new(self.text()?, self.text()?)?;
        let provenance =
            WorthQueryPackageReleaseProvenance::new(self.text()?, self.text()?, self.text()?)?;
        let requirements = self.decode_requirements()?;
        let signer_identity = self.text()?;
        let signature_protocol_identity = BoundaryProtocolIdentity::parse(self.text()?)
            .map_err(|_| Denial::new(Kind::InvalidEnvelopeProtocolIdentity))?;
        let signature_protocol_version = BoundaryProtocolVersion::try_new(self.input.u32()?)
            .map_err(|_| Denial::new(Kind::InvalidEnvelopeProtocolVersion))?;
        let signer = WorthQueryPackageReleaseSignerDescriptor::new(
            signer_identity,
            signature_protocol_identity,
            signature_protocol_version,
        )?;
        let descriptor = WorthQueryPackageReleaseEnvelopeDescriptor::new(
            build_metadata,
            release_metadata,
            provenance,
            signer,
        );
        let archive_length = usize::try_from(self.input.u64()?)
            .map_err(|_| Denial::new(Kind::EnvelopeArchiveByteBudgetExceeded))?;
        if u64::try_from(archive_length).unwrap_or(u64::MAX) > self.limits.maximum_archive_bytes() {
            return Err(Denial::new(Kind::EnvelopeArchiveByteBudgetExceeded));
        }
        let archive = self.input.take(archive_length)?.to_vec();
        if !self.input.is_finished() {
            return Err(Denial::new(Kind::TrailingBytes));
        }
        Ok(DecodedEnvelopeBody {
            archive,
            expected_package_identity,
            archive_checksum,
            descriptor,
            requirements,
        })
    }

    fn text(&mut self) -> Result<String, Denial> {
        let value = self.input.text()?;
        validate_descriptive_text(value)?;
        self.descriptive_text_bytes = self
            .descriptive_text_bytes
            .checked_add(u64::try_from(value.len()).unwrap_or(u64::MAX))
            .ok_or_else(|| Denial::new(Kind::EnvelopeDescriptiveTextByteBudgetExceeded))?;
        if self.descriptive_text_bytes > self.limits.maximum_descriptive_text_bytes() {
            return Err(Denial::new(Kind::EnvelopeDescriptiveTextByteBudgetExceeded));
        }
        Ok(value.to_owned())
    }

    fn decode_requirements(&mut self) -> Result<WorthQueryPackageReleaseRequirements, Denial> {
        let capabilities =
            self.decode_text_sequence(WorthQueryInstallationCapabilityFamily::new)?;
        let configuration =
            self.decode_text_sequence(WorthQueryInstallationConfigSectionFamily::new)?;
        let operating =
            self.decode_text_sequence(WorthQueryInstallationOperatingRequirement::new)?;
        let provider_count = self.claim_requirement_sequence(15)?;
        let mut execution_providers = Vec::with_capacity(provider_count);
        for _ in 0..provider_count {
            execution_providers.push(WorthQueryExecutionProviderRequirements::new(
                WorthQueryExecutionProviderFamily::new(self.text()?)
                    .map_err(|_| Denial::new(Kind::InvalidEnvelopeText))?,
                WorthQueryExecutionAccessProductFamily::new(self.text()?)
                    .map_err(|_| Denial::new(Kind::InvalidEnvelopeText))?,
                WorthQueryExecutionAllocatorFamily::new(self.text()?)
                    .map_err(|_| Denial::new(Kind::InvalidEnvelopeText))?,
            ));
        }
        WorthQueryPackageReleaseRequirements::from_untrusted_parts(
            capabilities,
            configuration,
            operating,
            execution_providers,
        )
    }

    fn decode_text_sequence<T>(
        &mut self,
        construct: impl Fn(String) -> T,
    ) -> Result<Vec<T>, Denial> {
        let count = self.claim_requirement_sequence(5)?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(construct(self.text()?));
        }
        Ok(values)
    }

    fn claim_requirement_sequence(
        &mut self,
        minimum_encoded_bytes_per_entry: u64,
    ) -> Result<usize, Denial> {
        let count = self.input.u32()?;
        self.requirements = self
            .requirements
            .checked_add(count)
            .ok_or_else(|| Denial::new(Kind::EnvelopeRequirementBudgetExceeded))?;
        if self.requirements > self.limits.maximum_requirements() {
            return Err(Denial::new(Kind::EnvelopeRequirementBudgetExceeded));
        }
        let minimum_encoded_bytes = u64::from(count)
            .checked_mul(minimum_encoded_bytes_per_entry)
            .ok_or_else(|| Denial::new(Kind::EnvelopeRequirementBudgetExceeded))?;
        if minimum_encoded_bytes > u64::try_from(self.input.remaining_len()).unwrap_or(u64::MAX) {
            return Err(Denial::new(Kind::Truncated));
        }
        usize::try_from(count).map_err(|_| Denial::new(Kind::EnvelopeRequirementBudgetExceeded))
    }
}
