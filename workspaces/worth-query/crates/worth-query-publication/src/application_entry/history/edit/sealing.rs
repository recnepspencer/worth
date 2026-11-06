//! Selection leaves the original completion outside the checked request handoff.
use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperation;

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
        let runtime = self.document.owner.application();
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
            let mut source = Mode::<S, I, A>::recording_source(completed, &phase, &current)?
                .ok_or(WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::Document(
                        WorthQueryDocumentHistoryDenial::ReceiptMismatch,
                    ),
                ))?;
            self.document
                .outputs
                .with_recording(&phase, |mut recording| {
                    let result = (|| {
                        if let Some(forest) = recording.as_deref_mut() {
                            if !super::recording::advance_output_recording::<
                                S,
                                H,
                                I,
                                P,
                                A,
                                Payload,
                            >(
                                &self.document.owner,
                                &self.document.payload,
                                &self.document.access,
                                &phase,
                                &current,
                                completed,
                                state.reservation.as_ref().expect("original reservation"),
                                state.candidate.as_ref().expect("actual candidate"),
                                forest,
                                &mut source,
                            )? {
                                return Ok(false);
                            }
                        }
                        let operation = WorthQueryDocumentHistoryOperation::new(
                            &selected,
                            &prepared.admission,
                            prepared.idempotency,
                        );
                        let reservation = state.reservation.take().expect("reserved record");
                        let candidate = state.candidate.take().expect("actual candidate");
                        let adopted = if let Some(forest) =
                            recording.as_ref().and_then(|forest| forest.completed())
                        {
                            let (source, required) = source.parts();
                            self.document
                                .owner
                                .adopt_recording_output_forest::<I::Binding>(
                                    &phase,
                                    &program_publication_access(),
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
                                HistoryRecordingSource::Source(receipt) => {
                                    self.document.owner.adopt_recorded_history::<I::Binding>(
                                        &phase,
                                        &program_publication_access(),
                                        &installed,
                                        operation,
                                        reservation,
                                        candidate,
                                        receipt,
                                    )
                                }
                                HistoryRecordingSource::RequiredOutputs { source, completed } => {
                                    self.document.owner.adopt_completed_history::<I::Binding>(
                                        &phase,
                                        &program_publication_access(),
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
                            Ok(step) => state.selected = Some(step),
                            Err(failure) => {
                                let (reservation, candidate, cause) = failure.into_parts();
                                state.reservation = Some(reservation);
                                state.candidate = Some(candidate);
                                return Err(WorthQueryHistoryEditDenial::History(
                                    WorthQueryApplicationHistoryDenial::Document(cause),
                                ));
                            }
                        }
                        if let Some(handoff) = &self.handoff {
                            handoff.pause();
                        }
                        self.document
                            .owner
                            .validate_record_delivery(&phase, &program_publication_access())
                            .map_err(|cause| {
                                WorthQueryHistoryEditDenial::History(
                                    WorthQueryApplicationHistoryDenial::Document(cause),
                                )
                            })
                            .map(|_| true)
                    })();
                    (result, state.selected.is_some())
                })
                .map_err(WorthQueryHistoryEditDenial::Outputs)?
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
