use super::super::{
    AdmittedPhysicalRecordFormat, AdmittedPhysicalRecordResidencyPolicy,
    AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
};
use super::residency_policy::canonical_residency_policy;
use crate::physical_runtime::PhysicalWorkProfileDeclaration;

pub struct PhysicalRecordInitialization {
    pub(in crate::physical_runtime::record_serving) read_protection:
        crate::physical_runtime::PhysicalReadProtectionPolicy,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) placement: AdmittedRecordPlacementPolicy,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) residency:
        AdmittedPhysicalRecordResidencyPolicy,
    pub(in crate::physical_runtime::record_serving) work_profile: PhysicalWorkProfileDeclaration,
    pub(in crate::physical_runtime::record_serving) durability:
        crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
}

impl PhysicalRecordInitialization {
    pub fn new(
        format: AdmittedPhysicalRecordFormat,
        placement: AdmittedRecordPlacementPolicy,
        access: AdmittedRecordAccessPolicy,
        durability: crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
    ) -> Self {
        Self {
            read_protection: crate::physical_runtime::PhysicalReadProtectionPolicy::default(),
            format,
            placement,
            access,
            residency: canonical_residency_policy(format),
            work_profile: PhysicalWorkProfileDeclaration::default(),
            durability,
        }
    }

    pub const fn with_residency_policy(
        mut self,
        policy: AdmittedPhysicalRecordResidencyPolicy,
    ) -> Self {
        self.residency = policy;
        self
    }

    pub fn with_physical_work_profile(mut self, profile: PhysicalWorkProfileDeclaration) -> Self {
        self.work_profile = profile;
        self
    }

    pub const fn with_read_protection_policy(
        mut self,
        policy: crate::physical_runtime::PhysicalReadProtectionPolicy,
    ) -> Self {
        self.read_protection = policy;
        self
    }
}

pub struct PhysicalRecordOpen {
    pub(in crate::physical_runtime::record_serving) read_protection:
        crate::physical_runtime::PhysicalReadProtectionPolicy,
    pub(in crate::physical_runtime::record_serving) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime::record_serving) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime::record_serving) residency:
        AdmittedPhysicalRecordResidencyPolicy,
    pub(in crate::physical_runtime::record_serving) work_profile: PhysicalWorkProfileDeclaration,
    pub(in crate::physical_runtime::record_serving) durability:
        crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
}

impl PhysicalRecordOpen {
    pub fn new(
        format: AdmittedPhysicalRecordFormat,
        access: AdmittedRecordAccessPolicy,
        durability: crate::physical_runtime::AdmittedPhysicalDurabilityPolicy,
    ) -> Self {
        Self {
            read_protection: crate::physical_runtime::PhysicalReadProtectionPolicy::default(),
            format,
            access,
            residency: canonical_residency_policy(format),
            work_profile: PhysicalWorkProfileDeclaration::default(),
            durability,
        }
    }

    pub const fn with_residency_policy(
        mut self,
        policy: AdmittedPhysicalRecordResidencyPolicy,
    ) -> Self {
        self.residency = policy;
        self
    }

    pub fn with_physical_work_profile(mut self, profile: PhysicalWorkProfileDeclaration) -> Self {
        self.work_profile = profile;
        self
    }

    pub const fn with_read_protection_policy(
        mut self,
        policy: crate::physical_runtime::PhysicalReadProtectionPolicy,
    ) -> Self {
        self.read_protection = policy;
        self
    }
}
