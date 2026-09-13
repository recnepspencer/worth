#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorruptionDenial {
    ReadmissionInputDoesNotMatchRequiredOutcome {
        required: super::classification::LayoutCorruptionClass,
    },
    FamilyBoundReadmissionWitnessRequired {
        family: crate::PhysicalArtifactFamily,
        source: super::readmission::LayoutReadmissionSource,
    },
    QuarantineObservationReadmissionEvidenceRequired {
        family: crate::PhysicalArtifactFamily,
    },
    AdmittedFamilyReadmissionAuthorityRequired {
        family: crate::PhysicalArtifactFamily,
    },
    SecurityScopeReadmissionMismatch {
        family: crate::PhysicalArtifactFamily,
        required: worth_store_security::StoreSecurityScopeIdentity,
        current: worth_store_security::StoreSecurityScopeIdentity,
    },
    QuarantineReadmissionRequired {
        family: crate::PhysicalArtifactFamily,
    },
    ImportReadmissionRequired {
        family: crate::PhysicalArtifactFamily,
    },
    NoForegroundReadAuthority {
        family: crate::PhysicalArtifactFamily,
    },
    UnexpectedQuarantineReadmissionClass {
        family: crate::PhysicalArtifactFamily,
        class: super::readmission::RecoveryLayoutReadmissionClass,
    },
}
