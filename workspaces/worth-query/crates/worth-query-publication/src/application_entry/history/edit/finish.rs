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
        terminal: crate::application_entry::WorthQueryTerminalSourceOperation<
            S,
            I,
            P,
            A,
            ApplicationSourceCompletion,
        >,
        closed: WorthQueryClosedOperation,
    },
}

pub struct WorthQueryHistoryEditFinish<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> where
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    pub outcome: Outcome<S, I, P, A>,
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
    pub fn finish(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        worth_execution::run_with_execution_stack(move || self.finish_on_execution_stack(request))
    }

    #[inline(never)]
    fn finish_on_execution_stack(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
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
            Phase::Selected { completed, step } => self.finish_selected(completed, step, request),
            Phase::AlreadyRecorded { operation, step } => {
                self.finish_already_recorded(operation, step, request)
            }
            Phase::Terminal {
                terminal,
                reservation,
                candidate,
            } => self.finish_terminal(terminal, reservation, candidate, request),
            Phase::TerminalCleanup {
                terminal,
                reservation,
                candidate,
            } => self.finish_terminal_cleanup(terminal, reservation, candidate, request),
            state => {
                self.phase = Some(state);
                Err((self, WorthQueryHistoryEditDenial::NotReady))
            }
        }
    }

    #[inline(never)]
    fn finish_selected(
        mut self,
        completed: Completed<S, I, P, A>,
        step: WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        let current = WorthQueryApplicationRequest {
            application: self.document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::completed_target(&completed),
        };
        match Mode::<S, I, A>::finish_recorded(completed, step.clone(), &current) {
            Ok(outcome) => Ok(WorthQueryHistoryEditFinish {
                document: self.document,
                outcome,
            }),
            Err((completed, cause)) => {
                self.phase = Some(Phase::Selected { completed, step });
                Err((self, WorthQueryHistoryEditDenial::Close(cause)))
            }
        }
    }

    #[inline(never)]
    fn finish_already_recorded(
        mut self,
        operation: Operation<S, I, P, A>,
        step: WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        let current = WorthQueryApplicationRequest {
            application: self.document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::target(&operation),
        };
        match Mode::<S, I, A>::finish_already(operation, step.clone(), &current) {
            Ok(outcome) => Ok(WorthQueryHistoryEditFinish {
                document: self.document,
                outcome,
            }),
            Err((operation, cause)) => {
                self.phase = Some(Phase::AlreadyRecorded { operation, step });
                Err((self, WorthQueryHistoryEditDenial::Close(cause)))
            }
        }
    }

    #[inline(never)]
    fn finish_terminal(
        mut self,
        terminal: Terminal<S, I, P, A>,
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        let current = WorthQueryApplicationRequest {
            application: self.document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::terminal_target(&terminal),
        };
        match Mode::<S, I, A>::close_terminal(terminal, &current) {
            Ok(terminal) => {
                self.phase = Some(Phase::TerminalCleanup {
                    terminal,
                    reservation,
                    candidate: WorthQueryRefusedCandidate::new(candidate),
                });
                self.finish(request)
            }
            Err((terminal, cause)) => {
                self.phase = Some(Phase::Terminal {
                    terminal,
                    reservation,
                    candidate,
                });
                Err((self, WorthQueryHistoryEditDenial::Close(cause)))
            }
        }
    }

    #[inline(never)]
    fn finish_terminal_cleanup(
        mut self,
        terminal: ClosedTerminal<S, I, P, A>,
        reservation: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryRefusedCandidate<'runtime, S>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        WorthQueryHistoryEditFinish<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryHistoryEditDenial),
    > {
        match super::discard::refuse_candidate(&self.document, request, reservation, candidate) {
            Ok(()) => Ok(WorthQueryHistoryEditFinish {
                document: self.document,
                outcome: Mode::<S, I, A>::finish_terminal(terminal),
            }),
            Err((reservation, candidate, denial)) => {
                self.phase = Some(Phase::TerminalCleanup {
                    terminal,
                    reservation,
                    candidate,
                });
                Err((self, denial))
            }
        }
    }
}
