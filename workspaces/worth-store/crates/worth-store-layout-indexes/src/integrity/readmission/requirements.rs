use super::authority::RecoveryLayoutReadmissionIdentity;

use crate::PhysicalArtifactFamily;

use super::super::quarantine::LayoutQuarantineWitness;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineReadmissionRequirement {
    pub(super) quarantine: LayoutQuarantineWitness,
    pub(super) identity: RecoveryLayoutReadmissionIdentity,
}

impl QuarantineReadmissionRequirement {
    pub(in crate::integrity) const fn new(
        quarantine: LayoutQuarantineWitness,
        identity: RecoveryLayoutReadmissionIdentity,
    ) -> Self {
        Self {
            quarantine,
            identity,
        }
    }
    pub const fn quarantine(&self) -> &LayoutQuarantineWitness {
        &self.quarantine
    }
    pub const fn identity(&self) -> &RecoveryLayoutReadmissionIdentity {
        &self.identity
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReadmissionRequirement {
    pub(super) family: crate::AdmittedPhysicalArtifactFamily,
    pub(super) identity: RecoveryLayoutReadmissionIdentity,
}

impl ImportReadmissionRequirement {
    pub(in crate::integrity) const fn new(
        family: crate::AdmittedPhysicalArtifactFamily,
        identity: RecoveryLayoutReadmissionIdentity,
    ) -> Self {
        Self { family, identity }
    }
    pub const fn family(&self) -> PhysicalArtifactFamily {
        self.family.lifecycle().declaration().family()
    }
    pub const fn admitted_family(&self) -> crate::AdmittedPhysicalArtifactFamily {
        self.family
    }
    pub const fn identity(&self) -> &RecoveryLayoutReadmissionIdentity {
        &self.identity
    }
}
