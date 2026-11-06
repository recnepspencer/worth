//! Actual document READ precedes lending recording's current control custody.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryReservation,
    WorthQueryExpectedProductBranch, WorthQueryHistoryNavigationDenial,
    WorthQueryHistoryOutputAuthority, WorthQueryHistoryRecordingOutputOrigin,
};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, NoApplicationMutationSource,
};
impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn with_document_history_recording_outputs<'runtime, 'scope, H, M, T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        origin: &std::sync::Arc<WorthQueryHistoryRecordingOutputOrigin>,
        callback: impl FnOnce(&WorthQueryHistoryOutputAuthority<'_, 'scope>) -> T,
    ) -> Result<T, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let completed = read
            .complete_metadata_read(self, phase, document, access)
            .map_err(WorthQueryHistoryNavigationDenial::Document)?;
        document.with_recording_output_authority(
            phase, access, &completed, installed, operation, reserved, candidate, origin, callback,
        )
    }
}
