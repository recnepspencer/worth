use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_installation::facade::{
    WorthQueryPortableDomainPackageIdentity, WorthQueryPortablePackageManifest,
    WorthQueryPortablePackageManifestVersion, WorthQueryPortablePackageRecordFamily,
};

use crate::binary_input::BinaryInput;
use crate::compatibility::{
    WorthQueryPackageArchiveCompatibilityProfile, WorthQueryPackageArchiveProtocolLayer,
};
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};
use crate::limits::WorthQueryPackageArchiveLimits;
use crate::protocol::MAGIC;

use super::protocol::MANIFEST_PAYLOAD_BYTES;

pub fn decode_manifest_frame(
    bytes: &[u8],
    limits: WorthQueryPackageArchiveLimits,
) -> Result<WorthQueryPortablePackageManifest, Denial> {
    let limits = limits.narrowed();
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limits.maximum_manifest_frame_bytes() {
        return Err(Denial::new(Kind::ManifestFrameByteBudgetExceeded));
    }
    let mut input = BinaryInput::new(bytes);
    if input.array::<8>()? != MAGIC {
        return Err(Denial::new(Kind::InvalidMagic));
    }
    require_supported_version(
        WorthQueryPackageArchiveProtocolLayer::Archive,
        input.u16()?,
        Kind::UnsupportedArchiveVersion,
    )?;
    if input.u32()? != MANIFEST_PAYLOAD_BYTES {
        return Err(Denial::new(Kind::InvalidManifestLength));
    }
    let version = WorthQueryPortablePackageManifestVersion::new(input.u16()?);
    require_supported_version(
        WorthQueryPackageArchiveProtocolLayer::Manifest,
        version.get(),
        Kind::UnsupportedManifestVersion,
    )?;
    let identity = WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(
        ARCHIVE_DECODE,
        input.array()?,
    );
    let record_count = input.u32()?;
    let canonical_source_bytes = input.u64()?;
    let logical_export_bytes = input.u64()?;
    if usize::from(input.u16()?) != WorthQueryPortablePackageRecordFamily::ALL.len() {
        return Err(Denial::new(Kind::InvalidFamilyCount));
    }
    let mut family_counts = [0_u32; WorthQueryPortablePackageRecordFamily::ALL.len()];
    for count in &mut family_counts {
        *count = input.u32()?;
    }
    if !input.is_finished() {
        return Err(Denial::new(Kind::TrailingBytes));
    }
    let manifest = WorthQueryPortablePackageManifest::from_untrusted_fields(
        ARCHIVE_DECODE,
        version,
        identity,
        record_count,
        canonical_source_bytes,
        logical_export_bytes,
        family_counts,
    );
    validate_claims(&manifest, limits)?;
    Ok(manifest)
}

fn require_supported_version(
    layer: WorthQueryPackageArchiveProtocolLayer,
    observed_version: u16,
    kind: Kind,
) -> Result<(), Denial> {
    WorthQueryPackageArchiveCompatibilityProfile::CURRENT
        .admit(layer, observed_version)
        .map_err(|compatibility| Denial::incompatible(kind, compatibility))
}

fn validate_claims(
    manifest: &WorthQueryPortablePackageManifest,
    limits: WorthQueryPackageArchiveLimits,
) -> Result<(), Denial> {
    if manifest.record_count() > limits.maximum_records() {
        return Err(Denial::new(Kind::RecordBudgetExceeded));
    }
    if manifest.logical_export_bytes() > limits.maximum_logical_bytes() {
        return Err(Denial::new(Kind::LogicalByteBudgetExceeded));
    }
    if manifest.canonical_source_bytes() > limits.maximum_canonical_work_bytes() {
        return Err(Denial::new(Kind::CanonicalWorkBudgetExceeded));
    }
    let total = WorthQueryPortablePackageRecordFamily::ALL
        .iter()
        .try_fold(0_u32, |total, family| {
            total.checked_add(manifest.family_count(*family))
        })
        .ok_or_else(|| Denial::new(Kind::InvalidFamilyCount))?;
    if total != manifest.record_count() {
        return Err(Denial::new(Kind::InvalidFamilyCount));
    }
    Ok(())
}
