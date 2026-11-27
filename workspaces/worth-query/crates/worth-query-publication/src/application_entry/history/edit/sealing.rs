//! Selection leaves the original completion outside the checked request handoff.
use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistoryOperation, WorthQueryRecordingKind,
};

impl<'runtime, S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>
    WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
    I::Binding: crate::application_entry::WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub(super) fn advance_selection(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        // The caller established that this request names the document's runtime.
        let runtime = request.application;
        let result = runtime.with_application_advancement(request.scope, |phase| {
            self.document
                .read_in_advancement(&phase, request)
                .map_err(WorthQueryHistoryEditDenial::History)?;
            let current = WorthQueryApplicationRequest {
                application: runtime,
                principal: request.principal,
                scope: request.scope,
                branch: self.document.owner.branch(),
            };
            let Some(Phase::Selecting(state)) = self.phase.as_mut() else {
                unreachable!()
            };
            let completed = state.completed.as_ref().expect("original SDK completion");
            let (prepared, selected) = Mode::<S, I, A>::prepare_completed(
                completed, &phase, &current,
            )
            .map_err(|cause| {
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Mutation(
                    cause,
                ))
            })?;
            let installed = runtime
                .installed_schema()
                .installed_history::<H>()
                .map_err(|cause| {
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::Installation(cause),
                    )
                })?;
            let source = Mode::<S, I, A>::recording_source(completed, &phase, &current)?.ok_or(
                WorthQueryHistoryEditDenial::History(WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::ReceiptMismatch,
                )),
            )?;
            let candidate = state.candidate.as_ref().expect("actual candidate").branch();
            let document = &mut self.document;
            let (outputs, payload, document_access) =
                (&document.outputs, &document.payload, &document.access);
            let handoff = &self.handoff;
            // The record, the candidate and the adopted step stay in the
            // edit's state; adoption takes the owner mutably and consumes
            // the source.
            let mut held = (
                &mut document.owner,
                &mut state.reservation,
                &mut state.candidate,
                &mut state.selected,
                Some(source),
            );
            let sealed = super::super::recording::seal(
                outputs,
                &phase,
                candidate,
                WorthQueryRecordingKind::Edit,
                &mut held,
                |held, forest| {
                    let (owner, reservation, candidate, _, source) = held;
                    super::recording::advance_output_recording::<S, H, I, P, A, Payload>(
                        owner,
                        payload,
                        document_access,
                        &phase,
                        &current,
                        completed,
                        reservation.as_ref().expect("original reservation"),
                        candidate.as_ref().expect("actual candidate"),
                        forest,
                        source.as_mut().expect("unconsumed recording source"),
                    )
                },
                |held, forest| {
                    let (owner, reservation_slot, candidate_slot, step, source) = held;
                    let operation = WorthQueryDocumentHistoryOperation::new(
                        &selected,
                        &prepared.admission,
                        prepared.idempotency,
                    );
                    let reservation = reservation_slot.take().expect("reserved record");
                    let candidate = candidate_slot.take().expect("actual candidate");
                    let source = source.take().expect("unconsumed recording source");
                    let access = program_publication_access();
                    let adopted = if let Some(forest) = forest {
                        let (source, required) = source.parts();
                        owner.adopt_recording_output_forest::<I::Binding>(
                            &phase,
                            &access,
                            &installed,
                            operation,
                            reservation,
                            candidate,
                            source,
                            required,
                            forest,
                        )
                    } else {
                        match source {
                            HistoryRecordingSource::Source(receipt) => owner
                                .adopt_recorded_history::<I::Binding>(
                                    &phase,
                                    &access,
                                    &installed,
                                    operation,
                                    reservation,
                                    candidate,
                                    receipt,
                                ),
                            HistoryRecordingSource::RecordedSource(receipt) => owner
                                .adopt_original_history::<I::Binding>(
                                    &phase,
                                    &access,
                                    &installed,
                                    operation,
                                    reservation,
                                    candidate,
                                    receipt,
                                ),
                            HistoryRecordingSource::RequiredOutputs { source, completed } => {
                                owner.adopt_completed_history::<I::Binding>(
                                    &phase,
                                    &access,
                                    &installed,
                                    operation,
                                    reservation,
                                    candidate,
                                    source,
                                    &completed,
                                )
                            }
                        }
                    };
                    match adopted {
                        Ok(adopted) => {
                            **step = Some(adopted);
                            Ok(())
                        }
                        Err(failure) => {
                            let (reservation, candidate, cause) = failure.into_parts();
                            **reservation_slot = Some(reservation);
                            **candidate_slot = Some(candidate);
                            Err(WorthQueryHistoryEditDenial::History(
                                WorthQueryApplicationHistoryDenial::Document(cause),
                            ))
                        }
                    }
                },
                |held| {
                    if let Some(handoff) = handoff {
                        handoff.pause();
                    }
                    held.0
                        .validate_record_delivery(&phase, &program_publication_access())
                        .map(|_| ())
                        .map_err(|cause| {
                            WorthQueryHistoryEditDenial::History(
                                WorthQueryApplicationHistoryDenial::Document(cause),
                            )
                        })
                },
            )
            .map_err(WorthQueryHistoryEditDenial::Outputs)?;
            sealed.map(|step| step.is_some())
        });
        if matches!(&self.phase, Some(Phase::Selecting(state)) if state.selected.is_some()) {
            let Some(Phase::Selecting(mut state)) = self.phase.take() else {
                unreachable!()
            };
            self.phase = Some(Phase::Selected {
                completed: state
                    .completed
                    .take()
                    .expect("original completion survives adoption"),
                step: state.selected.take().unwrap(),
            });
        }
        match result {
            Err(cause) => WorthQueryHistoryEditAdvance::Refused {
                edit: self,
                denial: WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::ExecutionRequest(cause),
                ),
            },
            Ok(Err(denial)) => WorthQueryHistoryEditAdvance::Refused { edit: self, denial },
            Ok(Ok(true)) => WorthQueryHistoryEditAdvance::Ready(self),
            Ok(Ok(false)) => WorthQueryHistoryEditAdvance::Progress(self),
        }
    }
}
