//! Lend an admitted original membership; never acquire a second local use.
use super::*;
use journal::QueryProgramSupportDenial as Denial;
use worth_foundational::facade::ProgramSupportObligationKind;

impl WorthQueryProgramSupportLifecycle {
    pub(in crate::domain_computation::primary_graph::program_occurrence) fn readmit_original_interpretation(
        &self,
        revision: &ApplicationProgramRevision,
        original: &worth_runtime_world::facade::WorldOriginalStateObservation<'_>,
    ) -> Result<WorthQueryProgramSupportInterpretation, Denial> {
        let state = self.state();
        if Self::pending_affects(&state, revision) {
            return Err(Denial::PublicationPending);
        }
        let journal = state.journal.as_ref().ok_or(Denial::SupportUnavailable)?;
        if original.program_revision().bytes() != revision.as_bytes()
            || original.attempt().owner().home() != journal.ledger.home()
        {
            return Err(Denial::ObligationSubjectMismatch);
        }
        let mut matching = journal.ledger.obligations().filter(|row| {
            row.kind == ProgramSupportObligationKind::OriginalInterpretation
                && row.original == original.attempt()
                && row.source.bytes() == revision.as_bytes()
                && row.target == row.source
        });
        let identity = matching.next().ok_or(Denial::SupportUnavailable)?.identity;
        if matching.next().is_some() {
            return Err(Denial::ObligationSubjectMismatch);
        }
        if !state.entries.contains_key(revision) {
            return Err(Denial::SupportUnavailable);
        }
        Ok(WorthQueryProgramSupportInterpretation {
            state: Arc::clone(&self.state),
            revision: *revision,
            membership: Arc::new(Mutex::new(InterpretationMembership::Durable(identity))),
        })
    }
}
