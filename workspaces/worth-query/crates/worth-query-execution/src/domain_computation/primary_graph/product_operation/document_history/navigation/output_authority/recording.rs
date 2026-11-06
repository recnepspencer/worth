//! Recording borrows actual C READ and operation authority, never NAV admission.
use super::super::super::{
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryRecordingOutputOrigin,
};
use super::*;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, NoApplicationMutationSource,
};
impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    super::super::super::WorthQueryDocumentHistory<'runtime, S, H>
{
    pub(in crate::domain_computation::primary_graph) fn with_recording_output_authority<
        'scope,
        M,
        T,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        read: &CompletedMetadataRead<'_, 'scope>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        reservation: &WorthQueryDocumentHistoryReservation,
        candidate: &crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch<
            'runtime,
            S,
        >,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        callback: impl FnOnce(&WorthQueryHistoryOutputAuthority<'_, 'scope>) -> T,
    ) -> Result<T, WorthQueryHistoryNavigationDenial>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        use WorthQueryHistoryNavigationDenial as Denial;
        let profile = origin.recording_profile();
        if !origin.matches_reservation(reservation, candidate)
            || profile.model != self.model
            || read.current_product().observation() != operation.selected.product().observation()
        {
            return Err(Denial::ForeignDocument);
        }
        let request = operation.admitted.retain_admitted_request();
        self.readmit_history_record::<M>(phase, access, installed, operation, reservation)
            .map_err(Denial::Document)?;
        let execution = phase
            .checked_execution_request_for(self.application().product_runtime())
            .map_err(Denial::Stopped)?;
        let selection = self
            .selection
            .lock()
            .retain_output_authority_lifetime(&execution)
            .map_err(Denial::Document)?;
        let callback_owner =
            OwnedHistoryAuthorityLifetime::reserve(&execution).map_err(|cause| {
                Denial::Document(
                    super::super::super::WorthQueryDocumentHistoryDenial::Recording(cause),
                )
            })?;
        let authority = WorthQueryHistoryOutputAuthority {
            phase,
            runtime: self.application().runtime.authority_identity().as_u64(),
            binding: self.binding.clone(),
            profile: profile.clone(),
            current: read.current_product().retained_clone(),
            restored: HistoryRestoredOutputBasis::recording(
                Arc::clone(origin.observation()),
                self.model.entity(),
                origin.has_original_required_outputs(),
            ),
            request,
            principal: read.principal(),
            selection,
            callback: callback_owner.retain(),
        };
        read.confirm(self, ()).map_err(Denial::Document)?;
        let result = callback(&authority);
        authority.confirm_read(phase, self.application(), authority.request(), result)
    }
}
