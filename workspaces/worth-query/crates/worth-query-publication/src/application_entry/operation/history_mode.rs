//! Installed action delivery selects the original history operation owner.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationActionDelivery;
mod driver;
mod fixed;
mod source;
pub(in crate::application_entry) use driver::{
    Driver as HistoryOperationDriver, HistoryOperationAdvance, HistoryRecordingSource,
};

pub(in crate::application_entry) type HistoryMode<S, I, A> =
    <Delivery<S, I, A> as ApplicationActionDelivery<S, Binding<S, I>>>::Mode;

/// Sealed availability of a history delivery profile. These are original SDK
/// owners and outcomes, not supplier callbacks or execution permission.
pub trait WorthQueryHistoryOperationMode<S, I, P, A>:
    driver::Driver<
    S,
    I,
    P,
    A,
    Self::Operation,
    Self::Completed,
    Self::Terminal,
    Self::ClosedTerminal,
    Self::Outcome,
>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
    type Operation;
    type Completed;
    type Terminal;
    type ClosedTerminal;
    type Outcome;
    type GroupClosing;
    type GroupCompletion;

    fn validate_group_outputs(
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryDenial>;

    fn begin_group_disposal(
        operation: Self::Operation,
    ) -> Result<Self::GroupClosing, (Self::Operation, WorthQueryOperationCloseDenial)>;
    fn begin_group_terminal_disposal(
        terminal: Self::Terminal,
    ) -> Result<Self::GroupClosing, (Self::Terminal, WorthQueryOperationCloseDenial)>;
    fn group_disposal_target(
        closing: &Self::GroupClosing,
    ) -> worth_query_execution::facade::product::WorthQueryProductBranch;
    fn close_group_disposal(
        closing: Self::GroupClosing,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<WorthQueryClosedOperation, (Self::GroupClosing, WorthQueryOperationCloseDenial)>;
    fn finish_group_member(
        completed: Self::Completed,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<Self::GroupCompletion, (Self::Completed, WorthQueryOperationCloseDenial)>;
    fn can_resume_group_cleanup(operation: &Self::Operation) -> bool;
}
