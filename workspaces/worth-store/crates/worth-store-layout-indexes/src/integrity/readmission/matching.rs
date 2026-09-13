use super::authority::{
    RecoveryLayoutReadmissionClass, RecoveryLayoutReadmissionIdentity,
    RecoveryLayoutReadmissionWitness,
};

use super::LayoutReadmissionSource;
use crate::integrity::CorruptionDenial;

pub(super) fn matches_identity(
    family: crate::AdmittedPhysicalArtifactFamily,
    expected: &RecoveryLayoutReadmissionIdentity,
    witness: &RecoveryLayoutReadmissionWitness,
) -> bool {
    witness.family_id() == family.family_id()
        && witness.identity() == expected
        && match witness.class() {
            RecoveryLayoutReadmissionClass::QuarantineRecovery
            | RecoveryLayoutReadmissionClass::ImportBoundaryReadmission => {
                witness.source_store_authority_identity() == Some(family.authority_identity())
                    && witness.source_security_scope_identity() == Some(family.security_identity())
            }
            RecoveryLayoutReadmissionClass::RebuildableDerivedObservation
            | RecoveryLayoutReadmissionClass::NoForegroundAuthority => false,
        }
}

pub(super) fn family_bound_denial(
    family: crate::PhysicalArtifactFamily,
    source: LayoutReadmissionSource,
) -> CorruptionDenial {
    CorruptionDenial::FamilyBoundReadmissionWitnessRequired { family, source }
}
