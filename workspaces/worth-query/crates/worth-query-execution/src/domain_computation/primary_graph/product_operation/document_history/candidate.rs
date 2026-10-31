//! Candidate creation uses the document's own stable runtime borrow and exact source.
use super::{
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryReservation,
};
use crate::{
    basis::WorthQueryProductBranchCreateError,
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryExpectedProductBranch,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};

#[derive(Debug)]
pub enum WorthQueryDocumentHistoryCandidateDenial {
    Admission(Denial),
    Creation(WorthQueryProductBranchCreateError),
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn create_history_candidate(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: &WorthQueryDocumentHistoryReservation,
    ) -> Result<
        WorthQueryExpectedProductBranch<'runtime, S>,
        WorthQueryDocumentHistoryCandidateDenial,
    > {
        self.validate_reservation(reserved)
            .map_err(WorthQueryDocumentHistoryCandidateDenial::Admission)?;
        let runtime = self.application();
        let selected = runtime
            .on_branch(self.branch())
            .select(phase)
            .map_err(|cause| {
                WorthQueryDocumentHistoryCandidateDenial::Admission(Denial::CandidateSelection(
                    cause,
                ))
            })?;
        if !self
            .expected
            .matches_observation(selected.product().observation())
        {
            return Err(WorthQueryDocumentHistoryCandidateDenial::Admission(
                Denial::HeadDiverged,
            ));
        }
        runtime
            .branches()
            .fork_expected(selected)
            .map_err(|cause| {
                WorthQueryDocumentHistoryCandidateDenial::Admission(Denial::CandidateSelection(
                    cause,
                ))
            })?
            .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
            .create(phase)
            .map_err(WorthQueryDocumentHistoryCandidateDenial::Creation)
    }
}
