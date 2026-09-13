use crate::PhysicalArtifactFamily;

use super::super::readmission::{
    RecoveryLayoutReadmissionClass, RecoveryLayoutReadmissionIdentity,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutQuarantineWitness {
    family: crate::AdmittedPhysicalArtifactFamily,
    observation_identity: RecoveryLayoutReadmissionIdentity,
    observation_class: RecoveryLayoutReadmissionClass,
}

impl LayoutQuarantineWitness {
    pub(in crate::integrity) fn from_observation(
        family: crate::AdmittedPhysicalArtifactFamily,
        observation_identity: RecoveryLayoutReadmissionIdentity,
        observation_class: RecoveryLayoutReadmissionClass,
    ) -> Self {
        Self {
            family,
            observation_identity,
            observation_class,
        }
    }

    pub const fn family(&self) -> PhysicalArtifactFamily {
        self.family.lifecycle().declaration().family()
    }

    pub const fn observation_identity(&self) -> &RecoveryLayoutReadmissionIdentity {
        &self.observation_identity
    }

    pub const fn observation_class(&self) -> RecoveryLayoutReadmissionClass {
        self.observation_class
    }

    pub const fn admitted_family(&self) -> crate::AdmittedPhysicalArtifactFamily {
        self.family
    }

    pub fn readmission_identity(&self) -> RecoveryLayoutReadmissionIdentity {
        self.observation_identity.clone()
    }
}
