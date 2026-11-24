//! The envelope's commit is the origin every bound output tree is recorded from.
use super::super::{
    source_evidence::WorthQueryHistorySourceEvidence, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryDenial as Denial, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryOutputProfile, WorthQueryHistoryRecordingOutputOrigin,
};
use crate::{
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
        WorthQueryExpectedProductBranch,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Capture the envelope's commit while it is still the candidate's
    /// endpoint. Later output work on the candidate cannot stand in for it.
    pub fn prepare_history_envelope_output_recording(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
        profile: &WorthQueryHistoryOutputProfile,
    ) -> Result<Arc<WorthQueryHistoryRecordingOutputOrigin>, Denial> {
        if !profile.is_output_forest() || !profile.matches_recording_model(&self.model) {
            return Err(Denial::ForeignModel);
        }
        let commit = self.prepare_envelope_binding(phase, reserved, candidate, receipt, None)?;
        let execution = phase
            .checked_execution_request_for(self.application().inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        WorthQueryHistoryRecordingOutputOrigin::capture(
            &execution,
            self.model.clone(),
            reserved,
            candidate.branch(),
            reserved.meaning,
            WorthQueryHistorySourceEvidence::Live(receipt),
            profile,
            commit.source,
            None,
        )
    }
}
