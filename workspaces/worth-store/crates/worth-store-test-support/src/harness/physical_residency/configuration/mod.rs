use std::num::{NonZeroU32, NonZeroU64};

use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy, ManifestEntryCapacity,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordPlacementPolicy,
};

mod record_publication;
mod recovery_planning;
#[cfg(test)]
mod successor_scope_pressure;

pub(super) use record_publication::record_publication_configuration;
pub(super) use recovery_planning::{
    compact_recovery_planning_configuration, dense_recovery_planning_configuration,
    recovery_planning_configuration,
};
#[cfg(test)]
pub(super) use successor_scope_pressure::successor_scope_pressure_configuration;

pub const SUCCESSOR_SCOPE_ALLOCATION_BYTES: u64 = FIXTURE_FRAME_BYTES;

pub(super) const FIXTURE_FRAME_BYTES: u64 = 16 * 1024;
pub(super) const FIXTURE_METADATA_BYTES: u64 = 256 * 1024;

pub(super) struct PhysicalResidencyStoreConfiguration {
    pub(super) format: AdmittedPhysicalRecordFormat,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) access: AdmittedRecordAccessPolicy,
    pub(super) residency: AdmittedPhysicalRecordResidencyPolicy,
}

pub(super) struct PhysicalResidencyStoreAdmissionBase {
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
}

impl PhysicalResidencyStoreAdmissionBase {
    pub(super) fn with_residency(
        self,
        residency: AdmittedPhysicalRecordResidencyPolicy,
    ) -> PhysicalResidencyStoreConfiguration {
        PhysicalResidencyStoreConfiguration {
            format: self.format,
            placement: self.placement,
            access: self.access,
            residency,
        }
    }

    pub(super) const fn format(&self) -> AdmittedPhysicalRecordFormat {
        self.format
    }

    pub(super) fn with_placement(mut self, placement: AdmittedRecordPlacementPolicy) -> Self {
        self.placement = placement;
        self
    }
}

pub(super) fn admitted_store_base() -> PhysicalResidencyStoreAdmissionBase {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let placement = PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(64).unwrap())
        .admit(format)
        .unwrap();
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    PhysicalResidencyStoreAdmissionBase {
        format,
        placement,
        access,
    }
}

pub(super) fn bytes(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}

pub(super) fn frames(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap()
}
