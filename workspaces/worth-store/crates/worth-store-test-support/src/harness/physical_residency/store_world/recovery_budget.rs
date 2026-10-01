//! A recovery world with one explicitly narrower, still owner-admitted scope.

use std::num::NonZeroU64;

use worth_store::physical_runtime::AdmittedPhysicalRecordFormat;

use super::{PhysicalResidencyStoreWorld, PhysicalResidencyStoreWorldConstructionFailure};

impl PhysicalResidencyStoreWorld {
    pub fn initialize_for_recovery_with_recovery_scope(
        label: &str,
        format: AdmittedPhysicalRecordFormat,
        manifest_capacity: u16,
        recovery_bytes: NonZeroU64,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            super::super::configuration::recovery_planning_configuration_with_recovery_scope(
                format,
                manifest_capacity,
                recovery_bytes,
            ),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }

    pub fn initialize_for_recovery_with_maintenance_scope(
        label: &str,
        format: AdmittedPhysicalRecordFormat,
        manifest_capacity: u16,
        maintenance_bytes: NonZeroU64,
    ) -> Result<Self, PhysicalResidencyStoreWorldConstructionFailure> {
        Self::initialize_with_configuration(
            label,
            super::super::configuration::recovery_planning_configuration_with_maintenance_scope(
                format,
                manifest_capacity,
                maintenance_bytes,
            ),
            NonZeroU64::new(16 * 1024 * 1024).unwrap(),
        )
    }
}
