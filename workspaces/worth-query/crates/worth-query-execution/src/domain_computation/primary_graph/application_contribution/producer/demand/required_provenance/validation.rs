//! Compare an actual required successor's issued mode with its effect call.

use super::*;

const MODE_SUBJECT: &str = "required successor commit mode differs from its issued mode";
const EDITION_SUBJECT: &str = "required successor installed producer edition moved";

impl DemandProgressionProvenance {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn validate_for_execution(
        &self,
        supplied: &WorthQueryProducerCommitAuthority,
        installed_edition: &InstalledProducerEdition,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(1)
            .map_err(|_| empty_work_denial())?;
        let Self::RequiredSuccessor(required) = self else {
            return Ok(());
        };
        required.validate_for_execution(supplied, installed_edition, admission)
    }
}

impl RequiredSuccessorProvenance {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn validate_for_execution(
        &self,
        supplied: &WorthQueryProducerCommitAuthority,
        installed_edition: &InstalledProducerEdition,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let subject_len = MODE_SUBJECT.len().max(EDITION_SUBJECT.len());
        let subject_backing = subject_len
            .checked_add(std::mem::size_of::<String>())
            .ok_or_else(empty_capacity_denial)?;
        admission
            .admit_read_scratch(
                u64::try_from(subject_backing).map_err(|_| empty_capacity_denial())?,
            )
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::WorkExhausted {
                    ..
                }
                | worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow => {
                    empty_work_denial()
                }
                _ => empty_capacity_denial(),
            })?;
        admission
            .charge_external_work(u64::try_from(subject_len).map_err(|_| empty_work_denial())?)
            .map_err(|_| empty_work_denial())?;
        // Read the two mode tags and the stored edition before selecting the
        // variable-width comparison below.
        admission
            .charge_external_work(3)
            .map_err(|_| empty_work_denial())?;
        let mode_work = match (&self.commit_authority, supplied) {
            (
                WorthQueryProducerCommitAuthority::SelectedProgram {
                    identity: expected, ..
                },
                WorthQueryProducerCommitAuthority::SelectedProgram {
                    identity: actual, ..
                },
            ) => {
                admission
                    .charge_external_work(2)
                    .map_err(|_| empty_work_denial())?;
                expected
                    .as_str()
                    .len()
                    .checked_add(actual.as_str().len())
                    .and_then(|work| work.checked_add(64 + 2))
                    .ok_or_else(empty_work_denial)?
            }
            _ => 2,
        };
        let edition_work = std::mem::size_of::<InstalledProducerEdition>()
            .checked_mul(2)
            .ok_or_else(empty_work_denial)?;
        let compare_work = mode_work
            .checked_add(edition_work)
            .ok_or_else(empty_work_denial)?;
        admission
            .charge_external_work(u64::try_from(compare_work).map_err(|_| empty_work_denial())?)
            .map_err(|_| empty_work_denial())?;
        let same_mode = match (&self.commit_authority, supplied) {
            (
                WorthQueryProducerCommitAuthority::Ordinary,
                WorthQueryProducerCommitAuthority::Ordinary,
            )
            | (
                WorthQueryProducerCommitAuthority::ProgramOutput,
                WorthQueryProducerCommitAuthority::ProgramOutput,
            ) => true,
            (
                WorthQueryProducerCommitAuthority::SelectedProgram {
                    identity: expected_identity,
                    revision: expected_revision,
                },
                WorthQueryProducerCommitAuthority::SelectedProgram {
                    identity: actual_identity,
                    revision: actual_revision,
                },
            ) => expected_identity == actual_identity && expected_revision == actual_revision,
            _ => false,
        };
        if !same_mode {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                MODE_SUBJECT,
            ));
        }
        if &self.installed_edition != installed_edition {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ProducerUnavailable,
                EDITION_SUBJECT,
            ));
        }
        Ok(())
    }
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "",
    )
}
