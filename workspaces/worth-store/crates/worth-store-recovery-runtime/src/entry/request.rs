use std::path::PathBuf;

use worth_store::physical_runtime::QualifiedPhysicalBackendProfile;

use super::{
    admission::admit_request, authority_binding::PhysicalRecoveryEntryPresentation,
    PhysicalRecoveryLimits, PhysicalRecoveryPlatformAuthority, PhysicalRecoveryRefusal,
    PhysicalRecoveryStaticConfiguration,
};
use crate::AdmittedPhysicalRecovery;

pub struct PhysicalRecoveryOpenRequest {
    pub(crate) presentation: PhysicalRecoveryEntryPresentation,
    pub(crate) authority: PhysicalRecoveryPlatformAuthority,
}

impl PhysicalRecoveryOpenRequest {
    pub fn declare(
        root: PathBuf,
        configuration: PhysicalRecoveryStaticConfiguration,
        backend_profile: QualifiedPhysicalBackendProfile,
        limits: PhysicalRecoveryLimits,
        authority: PhysicalRecoveryPlatformAuthority,
    ) -> Self {
        let presentation =
            authority.present_request(root, &backend_profile, &configuration, limits);
        Self {
            presentation,
            authority,
        }
    }

    pub fn admit(self) -> Result<AdmittedPhysicalRecovery, PhysicalRecoveryRefusal> {
        admit_request(self, None)
    }

    pub(crate) fn admit_with_process_yieldpoint(
        self,
        yieldpoint: worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint,
    ) -> Result<AdmittedPhysicalRecovery, PhysicalRecoveryRefusal> {
        admit_request(self, Some(yieldpoint))
    }
}
