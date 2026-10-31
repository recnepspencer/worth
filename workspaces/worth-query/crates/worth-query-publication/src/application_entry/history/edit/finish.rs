//! Business evidence transfers only after actual native cleanup and document selection.
use super::*;
use crate::application_entry::{WorthQueryClosedOperation, WorthQuerySourceCompletion};
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;

pub enum WorthQueryHistoryEditOutcome<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    Recorded {
        step: WorthQueryHistoryStep,
        completion:
            WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
        closed: WorthQueryClosedOperation,
    },
    AlreadyRecorded {
        step: WorthQueryHistoryStep,
        closed: WorthQueryClosedOperation,
    },
    NotRecorded {
        terminal: Terminal<S, I, P, A>,
        closed: WorthQueryClosedOperation,
    },
}

pub struct WorthQueryHistoryEditFinish<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    pub document: WorthQueryApplicationHistoryDocument<'runtime, S, H>,
    pub outcome: WorthQueryHistoryEditOutcome<S, I, P, A>,
}

impl<'runtime, S, H, I, P, A> WorthQueryHistoryEdit<'runtime, S, H, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    pub fn finish(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A>,
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
            Phase::Selected { completed, step } => {
                let current = WorthQueryApplicationRequest {
                    application: self.document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: completed.operation.history_target(),
                };
                match completed.close_with_evidence(&current) {
                    Ok((closed, completion)) => Ok(WorthQueryHistoryEditFinish {
                        document: self.document,
                        outcome: WorthQueryHistoryEditOutcome::Recorded {
                            step,
                            completion,
                            closed,
                        },
                    }),
                    Err((completed, cause)) => {
                        self.phase = Some(Phase::Selected { completed, step });
                        Err((self, WorthQueryHistoryEditDenial::Close(cause)))
                    }
                }
            }
            Phase::AlreadyRecorded { operation, step } => {
                let current = WorthQueryApplicationRequest {
                    application: self.document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: operation.history_target(),
                };
                match operation.close(&current) {
                    Ok(closed) => Ok(WorthQueryHistoryEditFinish {
                        document: self.document,
                        outcome: WorthQueryHistoryEditOutcome::AlreadyRecorded { step, closed },
                    }),
                    Err((operation, cause)) => {
                        self.phase = Some(Phase::AlreadyRecorded { operation, step });
                        Err((self, WorthQueryHistoryEditDenial::Close(cause)))
                    }
                }
            }
            Phase::Terminal {
                mut terminal,
                reservation,
                candidate,
            } => {
                let current = WorthQueryApplicationRequest {
                    application: self.document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: terminal.operation.history_target(),
                };
                match terminal.close_for_history(&current) {
                    Ok(closed) => {
                        self.phase = Some(Phase::TerminalCleanup {
                            terminal,
                            closed,
                            reservation,
                            candidate: DiscardedCandidate::Live {
                                head: candidate.into_owned_head(),
                                denial: None,
                            },
                        });
                        self.finish(request)
                    }
                    Err(cause) => {
                        self.phase = Some(Phase::Terminal {
                            terminal,
                            reservation,
                            candidate,
                        });
                        Err((self, WorthQueryHistoryEditDenial::Close(cause)))
                    }
                }
            }
            Phase::TerminalCleanup {
                terminal,
                closed,
                reservation,
                candidate,
            } => match super::discard::close_candidate(&self.document, request, candidate) {
                Ok(_) => {
                    self.document
                        .owner
                        .cancel_history_record(&program_publication_access(), reservation)
                        .unwrap_or_else(|_| {
                            unreachable!("the same terminal edit owns this reservation")
                        });
                    Ok(WorthQueryHistoryEditFinish {
                        document: self.document,
                        outcome: WorthQueryHistoryEditOutcome::NotRecorded { terminal, closed },
                    })
                }
                Err((candidate, denial)) => {
                    self.phase = Some(Phase::TerminalCleanup {
                        terminal,
                        closed,
                        reservation,
                        candidate,
                    });
                    Err((self, denial))
                }
            },
            state => {
                self.phase = Some(state);
                Err((self, WorthQueryHistoryEditDenial::NotReady))
            }
        }
    }
}
