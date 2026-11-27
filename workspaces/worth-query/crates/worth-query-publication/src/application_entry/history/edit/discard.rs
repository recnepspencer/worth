//! Explicit discard closes actual operation/candidate custody before releasing reserved slots.
use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;

/// One close attempt on a refused candidate, under its own advancement phase.
/// Retired cleanup retries under its own request, so it takes no phase.
pub(in crate::application_entry::history) fn close_candidate<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs,
>(
    document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    candidate: WorthQueryRefusedCandidate<'runtime, S>,
) -> Result<
    WorthQueryRefusedCandidate<'runtime, S>,
    (
        WorthQueryRefusedCandidate<'runtime, S>,
        WorthQueryHistoryEditDenial,
    ),
> {
    if !candidate.needs_phase() {
        return Ok(candidate.close(None));
    }
    let mut state = Some(candidate);
    let result =
        document
            .owner
            .application()
            .with_application_advancement(request.scope, |phase| {
                state = state.take().map(|candidate| candidate.close(Some(&phase)));
            });
    let state = state.expect("a close attempt keeps the candidate's custody");
    match result {
        Ok(()) => Ok(state),
        Err(cause) => Err((
            state,
            WorthQueryHistoryEditDenial::History(
                WorthQueryApplicationHistoryDenial::ExecutionRequest(cause),
            ),
        )),
    }
}

/// A refused edit closes its candidate, then releases its record. A
/// candidate that did not close stays with the record for the next retry.
pub(super) fn refuse_candidate<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs,
>(
    document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    reservation: WorthQueryDocumentHistoryReservation,
    candidate: WorthQueryRefusedCandidate<'runtime, S>,
) -> Result<
    (),
    (
        WorthQueryDocumentHistoryReservation,
        WorthQueryRefusedCandidate<'runtime, S>,
        WorthQueryHistoryEditDenial,
    ),
> {
    let candidate = match close_candidate(document, request, candidate) {
        Ok(candidate) => candidate,
        Err((candidate, denial)) => return Err((reservation, candidate, denial)),
    };
    match document.owner.refuse_history_record(
        &program_publication_access(),
        reservation,
        Some(candidate),
    ) {
        Ok(()) => Ok(()),
        Err((reservation, Some(candidate), None)) => Err((
            reservation,
            candidate,
            WorthQueryHistoryEditDenial::CandidateCleanup,
        )),
        Err(_) => unreachable!("the same refused edit owns this reservation"),
    }
}

impl<'runtime, S, H, I, P, A, Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>
    WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    Outputs: WorthQueryHistoryOutputsMode<S, H>,
{
    /// Deliberately discard an unselected candidate and its completed evidence.
    /// Native unpublished/indeterminate custody must settle before it can close.
    pub fn discard(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        worth_execution::run_with_execution_stack(move || self.discard_on_execution_stack(request))
    }

    #[inline(never)]
    fn discard_on_execution_stack(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return Err((
                self,
                WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::ForeignRuntime,
                ),
            ));
        }
        let state = self.phase.take().expect("owned phase");
        match state {
            Phase::Admission(state) | Phase::Forking(state) => self.discard_fork(request, state),
            Phase::Running {
                operation,
                reservation,
                candidate,
            } => {
                let current = WorthQueryApplicationRequest {
                    application: self.document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: Mode::<S, I, A>::target(&operation),
                };
                match Mode::<S, I, A>::close_operation(operation, &current) {
                    Ok(_) => {
                        self.phase = Some(Phase::Discarding {
                            candidate: WorthQueryRefusedCandidate::new(candidate),
                            reservation,
                        });
                        self.discard(request)
                    }
                    Err((operation, cause)) => {
                        self.phase = Some(Phase::Running {
                            operation,
                            reservation,
                            candidate,
                        });
                        Err((self, WorthQueryHistoryEditDenial::Close(cause)))
                    }
                }
            }
            Phase::Selecting(mut state) if state.selected.is_none() => {
                let branch = state
                    .candidate
                    .as_ref()
                    .expect("original candidate")
                    .branch();
                if let Err(cause) = self.document.outputs.discard_recording(request, branch) {
                    self.phase = Some(Phase::Selecting(state));
                    return Err((self, WorthQueryHistoryEditDenial::Outputs(cause)));
                }
                let completed = state.completed.take().unwrap();
                let current = WorthQueryApplicationRequest {
                    application: self.document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: Mode::<S, I, A>::completed_target(&completed),
                };
                match Mode::<S, I, A>::discard_completed(completed, &current) {
                    Ok(()) => {
                        self.phase = Some(Phase::Discarding {
                            candidate: WorthQueryRefusedCandidate::new(
                                state.candidate.take().unwrap(),
                            ),
                            reservation: state.reservation.take().unwrap(),
                        });
                        self.discard(request)
                    }
                    Err((completed, cause)) => {
                        state.completed = Some(completed);
                        self.phase = Some(Phase::Selecting(state));
                        Err((self, WorthQueryHistoryEditDenial::Close(cause)))
                    }
                }
            }
            Phase::Discarding {
                candidate,
                reservation,
            } => match refuse_candidate(&self.document, request, reservation, candidate) {
                Ok(()) => Ok(self.document),
                Err((reservation, candidate, denial)) => {
                    self.phase = Some(Phase::Discarding {
                        candidate,
                        reservation,
                    });
                    Err((self, denial))
                }
            },
            state => {
                let selected = matches!(
                    state,
                    Phase::Selected { .. } | Phase::AlreadyRecorded { .. } | Phase::Selecting(_)
                );
                self.phase = Some(state);
                Err((
                    self,
                    if selected {
                        WorthQueryHistoryEditDenial::AlreadySelected
                    } else {
                        WorthQueryHistoryEditDenial::NotReady
                    },
                ))
            }
        }
    }

    fn discard_fork(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        mut state: Forking<'runtime, S, I, P, A>,
    ) -> Result<
        WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        if let Err(cause) = ForkRecovery::release_in_request(&mut state.recovery, request) {
            self.phase = Some(Phase::Forking(state));
            return Err((self, cause));
        }
        let operation = state.operation.take().unwrap();
        let current = WorthQueryApplicationRequest {
            application: self.document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::target(&operation),
        };
        match Mode::<S, I, A>::close_operation(operation, &current) {
            Ok(_) => match (state.reservation.take(), state.candidate.take()) {
                (None, None) => Ok(self.document),
                (Some(reservation), None) => {
                    self.document
                        .owner
                        .refuse_history_record(&program_publication_access(), reservation, None)
                        .unwrap_or_else(|_| {
                            unreachable!("the same initial edit owns this reservation")
                        });
                    Ok(self.document)
                }
                (Some(reservation), Some(candidate)) => {
                    self.phase = Some(Phase::Discarding {
                        candidate: WorthQueryRefusedCandidate::new(candidate),
                        reservation,
                    });
                    self.discard(request)
                }
                (None, Some(_)) => unreachable!("storage precedes candidate effects"),
            },
            Err((operation, cause)) => {
                state.operation = Some(operation);
                self.phase = Some(if state.reservation.is_some() {
                    Phase::Forking(state)
                } else {
                    Phase::Admission(state)
                });
                Err((self, WorthQueryHistoryEditDenial::Close(cause)))
            }
        }
    }
}
