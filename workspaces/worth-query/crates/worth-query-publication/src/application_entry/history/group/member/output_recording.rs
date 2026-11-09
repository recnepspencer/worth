//! The final ordinary member lends its own admission and completion to the forest.
use super::*;
impl<S, I, P, A> Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub(super) fn recording_source_at<'a>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        endpoint: &crate::application_entry::WorthQueryApplicationReadObservation,
    ) -> Result<HistoryRecordingSource<'a>, WorthQueryHistoryGroupDenial> {
        let Self::Completed(Some(completed)) = self else {
            unreachable!("complete final member")
        };
        Mode::<S, I, A>::recording_source_at(completed, phase, request, endpoint)
            .map_err(WorthQueryHistoryGroupDenial::Member)?
            .ok_or(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::ReceiptMismatch,
                ),
            ))
    }
    pub(super) fn with_recording_outputs<'runtime, 'scope, H>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'runtime, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        group: &WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        origin: &std::sync::Arc<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin>,
        callback: &mut dyn FnMut(&worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputAuthority<'_, 'scope>) -> Result<bool, super::super::super::outputs::WorthQueryHistoryOutputDenial>,
    ) -> Result<bool, WorthQueryHistoryGroupDenial>
    where
        H: ApplicationHistoryBinding<S>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let Self::Completed(Some(completed)) = self else {
            unreachable!("complete final member")
        };
        let (prepared, selected) = Mode::<S, I, A>::prepare_completed(completed, phase, request)
            .map_err(|cause| {
                WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Mutation(
                    cause,
                ))
            })?;
        request
            .application
            .with_document_history_group_recording_outputs(
                phase,
                &program_publication_access(),
                owner,
                read,
                WorthQueryDocumentHistoryOperation::<S, I::Binding>::new(
                    &selected,
                    &prepared.admission,
                    prepared.idempotency,
                ),
                group,
                candidate,
                origin,
                |authority| callback(authority),
            )
            .map_err(|cause| {
                WorthQueryHistoryGroupDenial::Outputs(
                    super::super::super::outputs::WorthQueryHistoryOutputDenial::Navigation(cause),
                )
            })?
            .map_err(WorthQueryHistoryGroupDenial::Outputs)
    }
}
