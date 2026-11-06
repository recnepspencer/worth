//! Progress original additional trees and requalify the action at their one endpoint.
use super::*;
use crate::application_entry::history::outputs::RecordingForest;
// Borrow each real owner separately: the action receipt outlives this read,
// while reservation and candidate remain available to the atomic adoption.
pub(super) fn advance_output_recording<'runtime, 'a, S, H, I, P, A>(
    owner: &WorthQueryDocumentHistory<'runtime, S, H>,
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
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
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
    if !forest
        .advance(phase, &candidate_request)
        .map_err(WorthQueryHistoryEditDenial::Outputs)?
    {
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
