//! The edit delegates business execution to its original concrete SourceOnly owner.
use super::*;
use crate::application_entry::WorthQuerySourceOperationBinding;
use worth_query_declaration::facade::application_operation::NoApplicationMutationSource;

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
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    /// Read the document's selected head. Candidate work remains private until actual adoption.
    pub fn read(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<Query<S, H>, Rows<S, H>>,
        WorthQueryApplicationHistoryDenial,
    > {
        self.document.read(request)
    }

    pub fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        worth_execution::run_with_execution_stack(move || self.advance_on_execution_stack(request))
    }

    #[inline(never)]
    fn advance_on_execution_stack(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryHistoryEditAdvance<Self> {
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return WorthQueryHistoryEditAdvance::Refused {
                edit: self,
                denial: WorthQueryHistoryEditDenial::History(
                    WorthQueryApplicationHistoryDenial::ForeignRuntime,
                ),
            };
        }
        match self.posture() {
            WorthQueryHistoryEditPosture::AwaitingAdmission => {
                return self.advance_admission(request)
            }
            WorthQueryHistoryEditPosture::CreatingCandidate => return self.advance_fork(request),
            WorthQueryHistoryEditPosture::AwaitingSelection => {
                return self.advance_selection(request)
            }
            WorthQueryHistoryEditPosture::Running => {}
            _ => return WorthQueryHistoryEditAdvance::Ready(self),
        }
        let Some(Phase::Running {
            operation,
            reservation,
            candidate,
        }) = self.phase.take()
        else {
            unreachable!()
        };
        let current = WorthQueryApplicationRequest {
            application: self.document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: candidate.branch(),
        };
        match Mode::<S, I, A>::advance(operation, &current) {
            HistoryOperationAdvance::Progress(operation) => {
                self.phase = Some(Phase::Running {
                    operation,
                    reservation,
                    candidate,
                });
                WorthQueryHistoryEditAdvance::Progress(self)
            }
            HistoryOperationAdvance::Refused { operation, denial } => {
                self.phase = Some(Phase::Running {
                    operation,
                    reservation,
                    candidate,
                });
                WorthQueryHistoryEditAdvance::Refused { edit: self, denial }
            }
            HistoryOperationAdvance::Completed(completed) => {
                self.phase = Some(Phase::Selecting(Selecting {
                    completed: Some(completed),
                    reservation: Some(reservation),
                    candidate: Some(candidate),
                    selected: None,
                }));
                WorthQueryHistoryEditAdvance::Progress(self)
            }
            HistoryOperationAdvance::Terminal(terminal) => {
                self.phase = Some(Phase::Terminal {
                    terminal,
                    reservation,
                    candidate,
                });
                WorthQueryHistoryEditAdvance::Ready(self)
            }
        }
    }
}
