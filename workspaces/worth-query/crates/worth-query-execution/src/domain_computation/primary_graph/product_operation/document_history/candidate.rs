//! Candidate creation uses the document's own runtime custody and exact source.
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
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

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
        if !reserved
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
            .create_in(self.custody(), phase)
            .map_err(WorthQueryDocumentHistoryCandidateDenial::Creation)
    }
}
#[cfg(test)]
#[path = "candidate_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;
