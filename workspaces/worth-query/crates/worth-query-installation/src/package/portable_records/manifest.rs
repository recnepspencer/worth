//! Version and closure inventory for one typed logical package export.

use crate::package::WorthQueryPortableDomainPackageIdentity;

use super::WorthQueryPortablePackageRecordFamily;

pub const WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION: WorthQueryPortablePackageManifestVersion =
    WorthQueryPortablePackageManifestVersion::new(2);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortablePackageManifestVersion(u16);

impl WorthQueryPortablePackageManifestVersion {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPortablePackageManifest {
    version: WorthQueryPortablePackageManifestVersion,
    package_identity: WorthQueryPortableDomainPackageIdentity,
    record_count: u32,
    canonical_source_bytes: u64,
    logical_export_bytes: u64,
    family_counts: [u32; WorthQueryPortablePackageRecordFamily::ALL.len()],
}

impl WorthQueryPortablePackageManifest {
    pub(crate) fn new(
        package_identity: WorthQueryPortableDomainPackageIdentity,
        record_count: u32,
        canonical_source_bytes: u64,
        logical_export_bytes: u64,
        family_counts: [u32; WorthQueryPortablePackageRecordFamily::ALL.len()],
    ) -> Self {
        Self {
            version: WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
            package_identity,
            record_count,
            canonical_source_bytes,
            logical_export_bytes,
            family_counts,
        }
    }

    /// Construct one descriptive manifest received from an untrusted boundary.
    ///
    /// Construction performs no validation. Callers must pass the result to
    /// `WorthQueryPortablePackageReconstruction::begin` before any record set
    /// can close as a reconstruction candidate.
    pub const fn from_untrusted_fields(
        version: WorthQueryPortablePackageManifestVersion,
        package_identity: WorthQueryPortableDomainPackageIdentity,
        record_count: u32,
        canonical_source_bytes: u64,
        logical_export_bytes: u64,
        family_counts: [u32; WorthQueryPortablePackageRecordFamily::ALL.len()],
    ) -> Self {
        Self {
            version,
            package_identity,
            record_count,
            canonical_source_bytes,
            logical_export_bytes,
            family_counts,
        }
    }

    pub const fn version(&self) -> WorthQueryPortablePackageManifestVersion {
        self.version
    }

    pub const fn package_identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        &self.package_identity
    }

    pub const fn record_count(&self) -> u32 {
        self.record_count
    }

    pub const fn canonical_source_bytes(&self) -> u64 {
        self.canonical_source_bytes
    }

    pub const fn logical_export_bytes(&self) -> u64 {
        self.logical_export_bytes
    }

    pub const fn family_count(&self, family: WorthQueryPortablePackageRecordFamily) -> u32 {
        self.family_counts[family.index()]
    }

    pub(crate) const fn family_counts(
        &self,
    ) -> &[u32; WorthQueryPortablePackageRecordFamily::ALL.len()] {
        &self.family_counts
    }
}
