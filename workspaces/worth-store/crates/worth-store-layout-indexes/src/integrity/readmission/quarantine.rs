use super::authority::{RecoveryLayoutReadmissionClass, RecoveryLayoutReadmissionWitness};

use super::matching::matches_identity;
use super::{
    LayoutReadmissionSource, LayoutReadmissionWitness, QuarantineReadmissionCaseId,
    QuarantineReadmissionOutcome, QuarantineReadmissionRequirement,
};
use crate::integrity::{CorruptionDenial, LayoutReadmissionCounterSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarantineReadmission;

pub const fn quarantine_readmission() -> QuarantineReadmission {
    QuarantineReadmission
}

impl QuarantineReadmission {
    pub fn admit(
        self,
        required: QuarantineReadmissionRequirement,
        witness: RecoveryLayoutReadmissionWitness,
    ) -> QuarantineReadmissionOutcome {
        let quarantine = required.quarantine;
        let family = quarantine.family();
        let admitted_family = quarantine.admitted_family();
        match witness.class() {
            RecoveryLayoutReadmissionClass::QuarantineRecovery
                if matches_identity(admitted_family, &required.identity, &witness) =>
            {
                QuarantineReadmissionOutcome::readmitted(
                    LayoutReadmissionWitness::issue(
                        admitted_family,
                        LayoutReadmissionSource::QuarantineRecovery,
                        witness.identity(),
                    ),
                    LayoutReadmissionCounterSnapshot::new(1, 0, 1),
                )
            }
            RecoveryLayoutReadmissionClass::ImportBoundaryReadmission => {
                QuarantineReadmissionOutcome::denied(
                    CorruptionDenial::ImportReadmissionRequired { family },
                    QuarantineReadmissionCaseId::IMPORT_REQUIRED,
                    LayoutReadmissionCounterSnapshot::new(0, 0, 0),
                )
            }
            _ => QuarantineReadmissionOutcome::denied(
                CorruptionDenial::FamilyBoundReadmissionWitnessRequired {
                    family,
                    source: LayoutReadmissionSource::QuarantineRecovery,
                },
                QuarantineReadmissionCaseId::FAMILY_IDENTITY,
                LayoutReadmissionCounterSnapshot::new(1, 0, 0),
            ),
        }
    }
}
