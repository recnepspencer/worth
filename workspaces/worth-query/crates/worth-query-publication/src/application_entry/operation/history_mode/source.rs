//! Source completion retains the existing result and cleanup handoff unchanged.
use super::*;
use crate::application_entry::history::WorthQueryHistoryEditOutcome;
use crate::application_entry::operation::SourceAssessmentDriver;
use driver::*;
use worth_query_declaration::facade::{
    application_operation::NoApplicationMutationSource,
    application_program::ApplicationSourceCompletion,
};

type O<S, I, P, A> = WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>;
type C<S, I, P, A> = WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>;
type T<S, I, P, A> = WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>;
type CT<S, I, P, A> = (T<S, I, P, A>, WorthQueryClosedOperation);
type R<S, I, P, A> = WorthQueryHistoryEditOutcome<S, I, P, A>;

impl<S, I, P, A> WorthQueryHistoryOperationMode<S, I, P, A> for ApplicationSourceCompletion
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    type Operation = O<S, I, P, A>;
    type Completed = C<S, I, P, A>;
    type Terminal = T<S, I, P, A>;
    type ClosedTerminal = CT<S, I, P, A>;
    type Outcome = R<S, I, P, A>;
}
impl<S, I, P, A>
    Driver<S, I, P, A, O<S, I, P, A>, C<S, I, P, A>, T<S, I, P, A>, CT<S, I, P, A>, R<S, I, P, A>>
    for ApplicationSourceCompletion
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        O<S, I, P, A>,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        <Self as WorthQueryOperationModeFactory<S, I, P, A>>::construct(description, runtime)
    }
    fn prepare<'a>(
        operation: &'a O<S, I, P, A>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            crate::application_entry::mutation::authorization::PreparedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        <I::Binding as SourceAssessmentDriver<S>>::prepare_history(operation, phase, request)
    }
    fn prepare_completed<'a>(
        completed: &'a C<S, I, P, A>,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            crate::application_entry::mutation::authorization::PreparedMutation<S, I::Binding>,
            worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    > {
        Self::prepare(&completed.operation, phase, request)
    }
    fn target(
        operation: &O<S, I, P, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        operation.history_target()
    }
    fn completed_target(
        completed: &C<S, I, P, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        completed.operation.history_target()
    }
    fn terminal_target(
        terminal: &T<S, I, P, A>,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        terminal.operation.history_target()
    }
    fn bind_candidate(operation:O<S,I,P,A>, constraint:worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryOperationConstraint) -> Result<O<S,I,P,A>,(O<S,I,P,A>,worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial)>{
        operation.bind_history_candidate(constraint)
    }
    fn advance(
        operation: O<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> HistoryOperationAdvance<O<S, I, P, A>, C<S, I, P, A>, T<S, I, P, A>> {
        match operation.advance(request) {
            WorthQuerySourceOnlyAdvance::Progress(operation) => {
                HistoryOperationAdvance::Progress(operation)
            }
            WorthQuerySourceOnlyAdvance::Completed(completed) => {
                HistoryOperationAdvance::Completed(completed)
            }
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                HistoryOperationAdvance::Terminal(terminal)
            }
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => {
                HistoryOperationAdvance::Refused {
                    operation,
                    denial: crate::application_entry::history::WorthQueryHistoryEditDenial::Source(
                        denial,
                    ),
                }
            }
        }
    }
    fn recording_source(completed: &C<S, I, P, A>) -> Option<HistoryRecordingSource<'_>> {
        let (WorthQuerySourceCompletion::Fresh { receipt, .. }
        | WorthQuerySourceCompletion::Recovered { receipt, .. }) = completed.evidence();
        Some(HistoryRecordingSource::Source(receipt))
    }
    fn close_operation(
        operation: O<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (O<S, I, P, A>, WorthQueryOperationCloseDenial)> {
        operation.close(request)
    }
    fn discard_completed(
        completed: C<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), (C<S, I, P, A>, WorthQueryOperationCloseDenial)> {
        completed.close_with_evidence(request).map(drop)
    }
    fn finish_recorded(
        completed: C<S, I, P, A>,
        step: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<R<S, I, P, A>, (C<S, I, P, A>, WorthQueryOperationCloseDenial)> {
        completed
            .close_with_evidence(request)
            .map(|(closed, completion)| R::Recorded {
                step,
                completion,
                closed,
            })
    }
    fn finish_already(
        operation: O<S, I, P, A>,
        step: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<R<S, I, P, A>, (O<S, I, P, A>, WorthQueryOperationCloseDenial)> {
        operation
            .close(request)
            .map(|closed| R::AlreadyRecorded { step, closed })
    }
    fn close_terminal(
        mut terminal: T<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<CT<S, I, P, A>, (T<S, I, P, A>, WorthQueryOperationCloseDenial)> {
        match terminal.close_for_history(request) {
            Ok(closed) => Ok((terminal, closed)),
            Err(denial) => Err((terminal, denial)),
        }
    }
    fn retained_terminal(closed: &CT<S, I, P, A>) -> Option<&T<S, I, P, A>> {
        Some(&closed.0)
    }
    fn finish_terminal((terminal, closed): CT<S, I, P, A>) -> R<S, I, P, A> {
        R::NotRecorded { terminal, closed }
    }
}
