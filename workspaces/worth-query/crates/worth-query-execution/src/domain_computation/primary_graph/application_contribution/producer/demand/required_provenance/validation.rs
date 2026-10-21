//! Compare an actual required successor's issued mode with its effect call.

use super::*;

const MODE_SUBJECT: &str = "required successor commit mode differs from its issued mode";
const EDITION_SUBJECT: &str = "required successor installed producer edition moved";

impl DemandProgressionProvenance {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn validate_for_execution(
        &self,
        supplied: &WorthQueryProducerCommitAuthority,
        installed_edition: &InstalledProducerEdition,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Self::RequiredSuccessor(required) = self else {
            return Ok(());
        };
        required.validate_for_execution(supplied, installed_edition)
    }
}

impl RequiredSuccessorProvenance {
    pub(in crate::domain_computation::primary_graph::application_contribution::producer::demand) fn validate_for_execution(
        &self,
        supplied: &WorthQueryProducerCommitAuthority,
        installed_edition: &InstalledProducerEdition,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        // Read the two mode tags and the stored edition before selecting the
        // variable-width comparison below.

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
