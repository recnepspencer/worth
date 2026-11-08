//! Only installed mode implementations extract native evidence and delegate owners.
use super::*;
use crate::application_entry::history::WorthQueryHistoryEditDenial;
use crate::application_entry::mutation::authorization::PreparedMutation;
use worth_query_execution::{
    facade::primary_graph::{
        WorthQueryApplicationCommitReceipt, WorthQueryCompletedRequiredOutputs,
        WorthQuerySelectedProductOperation,
    },
    publication_boundary::document_history::{
        WorthQueryDocumentHistoryDenial, WorthQueryDocumentHistoryOperationConstraint,
        WorthQueryHistoryStep,
    },
};

pub enum HistoryRecordingSource<'a> {
    Source(&'a WorthQueryApplicationCommitReceipt),
    RequiredOutputs {
        source: &'a WorthQueryApplicationCommitReceipt,
        completed: WorthQueryCompletedRequiredOutputs,
    },
}
impl HistoryRecordingSource<'_> {
    pub(in crate::application_entry) fn parts(
        &self,
    ) -> (
        &WorthQueryApplicationCommitReceipt,
        Option<&WorthQueryCompletedRequiredOutputs>,
    ) {
        match self {
            Self::Source(source) => (source, None),
            Self::RequiredOutputs { source, completed } => (source, Some(completed)),
        }
    }
}
pub enum HistoryOperationAdvance<O, C, T> {
    Progress(O),
    Completed(C),
    Refused {
        operation: O,
        denial: WorthQueryHistoryEditDenial,
    },
    Terminal(T),
}

pub trait Driver<S, I, P, A, O, C, T, CT, R>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        O,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    >;
    fn prepare<'a>(
        operation: &'a O,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            PreparedMutation<S, I::Binding>,
            WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    >;
    fn prepare_completed<'a>(
        completed: &'a C,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        (
            PreparedMutation<S, I::Binding>,
            WorthQuerySelectedProductOperation<'a, S>,
        ),
        WorthQueryApplicationRequestMutationDenial,
    >;
    fn target(operation: &O) -> worth_query_execution::facade::product::WorthQueryProductBranch;
    fn completed_target(
        completed: &C,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch;
    fn terminal_target(
        terminal: &T,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch;
    fn bind_candidate(
        operation: O,
        constraint: WorthQueryDocumentHistoryOperationConstraint,
    ) -> Result<O, (O, WorthQueryDocumentHistoryDenial)>;
    fn advance(
        operation: O,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> HistoryOperationAdvance<O, C, T>;
    fn recording_source<'a>(
        completed: &'a C,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<Option<HistoryRecordingSource<'a>>, WorthQueryHistoryEditDenial>;
    fn recording_source_at<'a>(
        completed: &'a C,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        observation: &crate::application_entry::WorthQueryApplicationReadObservation,
    ) -> Result<Option<HistoryRecordingSource<'a>>, WorthQueryHistoryEditDenial>;
    fn close_operation(
        operation: O,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (O, WorthQueryOperationCloseDenial)>;
    fn discard_completed(
        completed: C,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), (C, WorthQueryOperationCloseDenial)>;
    fn finish_recorded(
        completed: C,
        step: WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<R, (C, WorthQueryOperationCloseDenial)>;
    fn finish_already(
        operation: O,
        step: WorthQueryHistoryStep,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<R, (O, WorthQueryOperationCloseDenial)>;
    fn close_terminal(
        terminal: T,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<CT, (T, WorthQueryOperationCloseDenial)>;
    fn retained_terminal(closed: &CT) -> Option<&T>;
    fn finish_terminal(closed: CT) -> R;
}
