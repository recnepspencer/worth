//! Explicit discard closes actual operation/candidate custody before releasing reserved slots.
use super::*;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;
use worth_query_execution::facade::product::WorthQueryOwnedProductHeadCloseFailure;

pub(in crate::application_entry::history) fn close_candidate<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
    Outputs,
>(
    document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    candidate: DiscardedCandidate<'runtime, S>,
) -> Result<
    DiscardedCandidate<'runtime, S>,
    (DiscardedCandidate<'runtime, S>, WorthQueryHistoryEditDenial),
> {
    match candidate {
        done @ (DiscardedCandidate::ClosedDirect { .. }
        | DiscardedCandidate::ClosedCleanup { .. }) => Ok(done),
        DiscardedCandidate::Retired(failure) => match failure.into_cleanup().retry() {
            Ok(receipt) => Ok(DiscardedCandidate::ClosedCleanup { _receipt: receipt }),
            Err(failure) => Err((
                DiscardedCandidate::Retired(failure),
                WorthQueryHistoryEditDenial::CandidateCleanup,
            )),
        },
        DiscardedCandidate::Live { head, .. } => {
            let runtime = document.owner.application();
            let mut state = Some(DiscardedCandidate::Live { head, denial: None });
            let result = runtime.with_application_advancement(request.scope, |phase| {
                let Some(DiscardedCandidate::Live { head, .. }) = state.take() else {
                    unreachable!()
                };
                state = Some(match head.close(&phase) {
                    Ok(receipt) => DiscardedCandidate::ClosedDirect { _receipt: receipt },
                    Err(WorthQueryOwnedProductHeadCloseFailure::Retained { head, denial }) => {
                        DiscardedCandidate::Live {
                            head,
                            denial: Some(denial),
                        }
                    }
                    Err(WorthQueryOwnedProductHeadCloseFailure::Retired { failure, .. }) => {
                        DiscardedCandidate::Retired(failure)
                    }
                });
            });
            let state =
                state.expect("close retains live or retired custody, or its actual closed receipt");
            match result {
                Err(cause) => Err((
                    state,
                    WorthQueryHistoryEditDenial::History(
                        WorthQueryApplicationHistoryDenial::ExecutionRequest(cause),
                    ),
                )),
                Ok(()) if matches!(state, DiscardedCandidate::ClosedDirect { .. }) => Ok(state),
                Ok(()) => Err((state, WorthQueryHistoryEditDenial::CandidateCleanup)),
            }
        }
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
{
    /// Deliberately discard an unselected candidate and its completed evidence.
    /// Native unpublished/indeterminate custody must settle before it can close.
    pub fn discard(
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
                            candidate: DiscardedCandidate::Live {
                                head: candidate.into_owned_head(),
                                denial: None,
                            },
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
                            candidate: DiscardedCandidate::Live {
                                head: state.candidate.take().unwrap().into_owned_head(),
                                denial: None,
                            },
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
            } => match close_candidate(&self.document, request, candidate) {
                Ok(_) => {
                    self.document
                        .owner
                        .cancel_history_record(&program_publication_access(), reservation)
                        .unwrap_or_else(|_| {
                            unreachable!("the same discarded edit owns this reservation")
                        });
                    Ok(self.document)
                }
                Err((candidate, denial)) => {
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
        if let Some(recovery) = state.recovery.take()
            && let Err(next) = recovery.release()
        {
            state.recovery = Some(next);
            self.phase = Some(Phase::Forking(state));
            return Err((self, WorthQueryHistoryEditDenial::ForkRecovery));
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
                        .cancel_history_record(&program_publication_access(), reservation)
                        .unwrap_or_else(|_| {
                            unreachable!("the same initial edit owns this reservation")
                        });
                    Ok(self.document)
                }
                (Some(reservation), Some(candidate)) => {
                    self.phase = Some(Phase::Discarding {
                        candidate: DiscardedCandidate::Live {
                            head: candidate.into_owned_head(),
                            denial: None,
                        },
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
