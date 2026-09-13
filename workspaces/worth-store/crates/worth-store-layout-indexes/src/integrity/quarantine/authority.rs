use super::super::readmission::RecoveryLayoutReadmissionClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::integrity) enum LayoutQuarantineAuthorityClass {
    DerivedProjectionDamage,
    AuthoritativeQuarantineRequired,
}

pub(in crate::integrity) fn classify_quarantine_authority(
    class: RecoveryLayoutReadmissionClass,
) -> LayoutQuarantineAuthorityClass {
    match class {
        RecoveryLayoutReadmissionClass::RebuildableDerivedObservation => {
            LayoutQuarantineAuthorityClass::DerivedProjectionDamage
        }
        RecoveryLayoutReadmissionClass::QuarantineRecovery
        | RecoveryLayoutReadmissionClass::ImportBoundaryReadmission
        | RecoveryLayoutReadmissionClass::NoForegroundAuthority => {
            LayoutQuarantineAuthorityClass::AuthoritativeQuarantineRequired
        }
    }
}
