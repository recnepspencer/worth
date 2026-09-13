use worth_store_physical_backend::ArtifactTreeFailure;
use worth_store_physical_format::{
    CurrentRootCatalogEntry, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
};

use super::super::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, RecordByteLimit,
    RecordPublicationResidueObservation,
};

pub(in crate::physical_runtime) struct PhysicalRecordBootstrapOwner {
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) current_root: CurrentRootCatalogEntry,
    pub(in crate::physical_runtime::record_serving) observed_staging_residue: bool,
}

pub(in crate::physical_runtime) struct RecordServingState {
    pub(in crate::physical_runtime) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime) current_root: DurablePhysicalRootManifest,
    pub(in crate::physical_runtime) previous_root: Option<DurablePhysicalRootManifest>,
    pub(in crate::physical_runtime) publication_residue: RecordPublicationResidueObservation,
    pub(in crate::physical_runtime) free_space: DurableFreeSpaceManifestHeader,
    pub(in crate::physical_runtime) root_protocol_counters:
        crate::physical_runtime::RootProtocolRouteCounters,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BootstrapCatalogReadLimits {
    catalog_bytes: u32,
    current_root_bytes: RecordByteLimit,
    current_root_entries: u16,
}

impl BootstrapCatalogReadLimits {
    pub(in crate::physical_runtime::record_serving) fn for_format(
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
    ) -> Self {
        let page_bytes = format.declaration().page_size().bytes();
        let current_root_bytes =
            RecordByteLimit::new(access.transfer_limit().get().min(page_bytes))
                .expect("an admitted page size is nonzero");
        let current_root_entries =
            worth_store_physical_format::maximum_current_root_entries(format.declaration());
        Self {
            catalog_bytes: worth_store_physical_format::BOOTSTRAP_CATALOG_BYTES as u32,
            current_root_bytes,
            current_root_entries,
        }
    }

    pub const fn catalog_bytes(self) -> u32 {
        self.catalog_bytes
    }

    pub const fn current_root_bytes(self) -> RecordByteLimit {
        self.current_root_bytes
    }

    pub const fn current_root_entries(self) -> u16 {
        self.current_root_entries
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordBootstrapDenial {
    IdentityEntropyUnavailable,
    ConfigurationMismatch,
    RecordFamilyAlreadyExists,
    RecordFamilyAbsent,
    AmbiguousRecordFamilyResidue,
    CatalogMissing,
    CatalogDamaged,
    UnsupportedPhysicalRecordFormat(UnsupportedPhysicalRecordFormat),
    PhysicalRecordFormatMismatch(PhysicalRecordFormatMismatch),
    CurrentRootDamaged,
    FreeSpaceManifestDamaged,
    BackendUnavailable(ArtifactTreeFailure),
    ResidencyUnavailable(super::super::PhysicalRecordResidencyFailure),
}

impl RecordBootstrapDenial {
    pub(in crate::physical_runtime::record_serving) fn from_residency(
        denial: worth_store_buffer_pool::PhysicalResidencyDenial,
    ) -> Self {
        Self::ResidencyUnavailable(denial.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedPhysicalRecordFormat {
    reason: worth_store_physical_format::PhysicalRecordFormatDenial,
}

impl UnsupportedPhysicalRecordFormat {
    pub(in crate::physical_runtime::record_serving) const fn new(
        reason: worth_store_physical_format::PhysicalRecordFormatDenial,
    ) -> Self {
        Self { reason }
    }
    pub const fn reason(self) -> worth_store_physical_format::PhysicalRecordFormatDenial {
        self.reason
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecordFormatMismatch {
    expected: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    persisted: worth_store_physical_format::PhysicalRecordFormatDeclaration,
}

impl PhysicalRecordFormatMismatch {
    pub(in crate::physical_runtime::record_serving) const fn new(
        expected: worth_store_physical_format::PhysicalRecordFormatDeclaration,
        persisted: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    ) -> Self {
        Self {
            expected,
            persisted,
        }
    }
    pub const fn expected(self) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.expected
    }
    pub const fn persisted(self) -> worth_store_physical_format::PhysicalRecordFormatDeclaration {
        self.persisted
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordBootstrapFailure {
    Backend(ArtifactTreeFailure),
    FormatEncoding,
    PublishedRootReadmission(RecordBootstrapDenial),
    PublishedRootStale(RecordServingStaleReason),
    PublishedRootRebindRequired(RecordServingRebindReason),
    SignalConstruction(crate::physical_runtime::PhysicalSignalConstructionFailure),
}

pub(in crate::physical_runtime::record_serving) enum BootstrapTransitionFailure {
    Denied(RecordBootstrapDenial),
    Stale(RecordServingStaleReason),
    RebindRequired(RecordServingRebindReason),
    Failed(RecordBootstrapFailure),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordServingStaleReason {
    CatalogSelectedRootGenerationMismatch,
    FreeSpaceGenerationMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordServingRebindReason {
    StoreIdentityMismatch,
    PhysicalDurabilityStoreMismatch,
    PhysicalDurabilityAdmissionBasisMismatch,
}

pub(in crate::physical_runtime::record_serving) fn backend_before_effect(
    failure: ArtifactTreeFailure,
) -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Denied(RecordBootstrapDenial::BackendUnavailable(failure))
}

pub(in crate::physical_runtime::record_serving) fn backend_after_effect(
    failure: ArtifactTreeFailure,
) -> BootstrapTransitionFailure {
    BootstrapTransitionFailure::Failed(RecordBootstrapFailure::Backend(failure))
}
