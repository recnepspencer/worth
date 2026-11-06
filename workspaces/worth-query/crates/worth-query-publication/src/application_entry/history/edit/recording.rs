//! Progress original additional trees and requalify the action at their one endpoint.
use super::*;
use crate::application_entry::history::outputs::RecordingForest;
// Borrow each real owner separately: the action receipt outlives this read,
// while reservation and candidate remain available to the atomic adoption.
pub(super) fn advance_output_recording<'runtime, 'a, S, H, I, P, A, Payload>(
    owner: &WorthQueryDocumentHistory<'runtime, S, H>,
    payload: &Payload,
    access: &H::Access,
    phase: &WorthQueryAdvancementPhase<'_>,
    current: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    completed: &'a Completed<S, I, P, A>,
    reservation: &WorthQueryDocumentHistoryReservation,
    candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
    forest: &mut RecordingForest<S>,
    source: &mut HistoryRecordingSource<'a>,
) -> Result<bool, WorthQueryHistoryEditDenial>
where
    S: ApplicationSchema + 'static,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = worth_query_declaration::facade::application_operation::NoApplicationMutationSource>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    if forest.needs_origin() {
        let (receipt, required) = source.parts();
        let origin = owner
            .prepare_history_output_recording(
                phase,
                &program_publication_access(),
                reservation,
                candidate,
                receipt,
                required,
                forest.profile(),
            )
            .map_err(|cause| {
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                    cause,
                ))
            })?;
        forest
            .bind_origin(origin)
            .map_err(WorthQueryHistoryEditDenial::Outputs)?;
    }
    let candidate_request = WorthQueryApplicationRequest {
        application: owner.application(),
        principal: current.principal,
        scope: current.scope,
        branch: candidate.branch(),
    };
    let installed = owner
        .application()
        .installed_schema()
        .installed_history::<H>()
        .map_err(|cause| {
            WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Installation(
                cause,
            ))
        })?;
    let origin = std::sync::Arc::clone(forest.origin().expect("bound original recording"));
    let complete=payload.with_read(current.query(access.clone()),phase,None,
        |runtime,binding,external,plan| {
            let read=worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryRead::new(&installed,binding,external,plan);
            Ok((|| {
            let (prepared,selected)=Mode::<S,I,A>::prepare_completed(completed,phase,current)
                .map_err(|cause|WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Mutation(cause)))?;
            let operation=worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperation::<S, I::Binding>::new(&selected,&prepared.admission,prepared.idempotency);
            runtime.with_document_history_recording_outputs(phase,&program_publication_access(),owner,read,operation,reservation,candidate,&origin,
                |authority|forest.advance(phase,&candidate_request,authority))
                .map_err(|cause|WorthQueryHistoryEditDenial::Outputs(crate::application_entry::history::outputs::WorthQueryHistoryOutputDenial::Navigation(cause)))?
                .map_err(WorthQueryHistoryEditDenial::Outputs)
            })())
        }).map_err(|cause|WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Query(cause)))??;
    if !complete {
        return Ok(false);
    }
    let endpoint = crate::application_entry::WorthQueryApplicationReadObservation::new(
        std::sync::Arc::clone(forest.completed().expect("complete forest").observation()),
    );
    *source = Mode::<S, I, A>::recording_source_at(completed, phase, current, &endpoint)?.ok_or(
        WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
            WorthQueryDocumentHistoryDenial::ReceiptMismatch,
        )),
    )?;
    forest
        .drain_for_selection(phase, candidate.branch())
        .map_err(WorthQueryHistoryEditDenial::Outputs)?;

    Ok(true)
}
