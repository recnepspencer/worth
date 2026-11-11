//! Candidate-local disposal leaves the authentic publication and Home definition proof.
use super::PreparedCommitPublicationExecution;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;

impl PreparedCommitPublicationExecution {
    pub(crate) fn discard_prepared_index_publication(&mut self) -> Result<(), Denial> {
        let prepared = self
            .prepared_indexes
            .take()
            .ok_or(Denial::CandidateUnavailable)?;
        // Drop the actual unpublished payload and its existing original-request
        // backing together. No catalog installation or authority changes occur.
        drop(prepared);
        Ok(())
    }
}
