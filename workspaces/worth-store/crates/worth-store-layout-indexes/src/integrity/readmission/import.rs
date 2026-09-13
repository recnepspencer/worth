use super::authority::{RecoveryLayoutReadmissionClass, RecoveryLayoutReadmissionWitness};

use super::matching::{family_bound_denial, matches_identity};
use super::{
    ImportReadmissionCaseId, ImportReadmissionOutcome, ImportReadmissionRequirement,
    LayoutReadmissionSource, LayoutReadmissionWitness,
};
use crate::integrity::{CorruptionDenial, LayoutReadmissionCounterSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportReadmission;

pub const fn import_readmission() -> ImportReadmission {
    ImportReadmission
}

impl ImportReadmission {
    pub fn admit(
        self,
        required: ImportReadmissionRequirement,
        witness: RecoveryLayoutReadmissionWitness,
    ) -> ImportReadmissionOutcome {
        let family = required.family;
        let family_declaration = required.family();
        match witness.class() {
            RecoveryLayoutReadmissionClass::ImportBoundaryReadmission
                if matches_identity(family, &required.identity, &witness) =>
            {
                ImportReadmissionOutcome::readmitted(
                    LayoutReadmissionWitness::issue(
                        family,
                        LayoutReadmissionSource::TerminalImport,
                        witness.identity(),
                    ),
                    LayoutReadmissionCounterSnapshot::new(1, 0, 1),
                )
            }
            RecoveryLayoutReadmissionClass::QuarantineRecovery => ImportReadmissionOutcome::denied(
                CorruptionDenial::QuarantineReadmissionRequired {
                    family: family_declaration,
                },
                ImportReadmissionCaseId::QUARANTINE_REQUIRED,
                LayoutReadmissionCounterSnapshot::new(0, 0, 0),
            ),
            _ => ImportReadmissionOutcome::denied(
                family_bound_denial(family_declaration, LayoutReadmissionSource::TerminalImport),
                ImportReadmissionCaseId::FAMILY_IDENTITY,
                LayoutReadmissionCounterSnapshot::new(1, 0, 0),
            ),
        }
    }
}
