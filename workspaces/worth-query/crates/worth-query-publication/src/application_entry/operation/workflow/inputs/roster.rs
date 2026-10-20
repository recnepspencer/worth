//! A sealed heterogeneous list delegates custody; it contains no business cursor.
use super::pack::WorthQueryWorkflowInputDenial;
use crate::application_entry::{
    WorthQueryApplicationRequest, WorthQueryClosedOperation, WorthQueryOperationCloseDenial,
    WorthQueryOperationDescription, WorthQueryWorkflowOperationAdvance,
};
use worth_query_declaration::facade::{
    application_operation::ApplicationMutationIntent,
    application_program::{ApplicationProgramDefinition, ApplicationWorkflowSpec},
    application_schema::ApplicationSchema,
};
use worth_query_execution::{
    facade::application_installation::{
        WorthQueryProgramApplicationRuntime, WorthQueryWorkflowVocabulary,
    },
    publication_boundary::workflow_advance::RequiredWorkflowOperation,
};
mod end;
mod member;
pub(super) mod sealed {
    pub trait List {}
}
/// Empty finite input roster; a native requirement cannot select an operation here.
pub struct WorthQueryWorkflowInputEnd;
/// One original typed description and installed action, prepended by `with`.
/// Fields are private so consumers cannot forge or refill transferred custody.
pub struct WorthQueryWorkflowInputNode<S: ApplicationSchema, I: ApplicationMutationIntent<S>, A, T>
{
    pub(super) description: Option<WorthQueryOperationDescription<S, I>>,
    pub(super) target: Option<
        worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity,
    >,
    pub(super) tail: T,
    pub(super) action: std::marker::PhantomData<fn() -> A>,
}
/// A structural sum of actual native owners, not an independently performed result.
pub enum WorthQueryWorkflowOperationChoice<H, T> {
    Here(H),
    Other(T),
}
impl<H, T> WorthQueryWorkflowOperationChoice<H, T> {
    fn close_with<C, E>(
        self,
        head: impl FnOnce(H) -> Result<C, (H, E)>,
        tail: impl FnOnce(T) -> Result<C, (T, E)>,
    ) -> Result<C, (Self, E)> {
        match self {
            Self::Here(owner) => head(owner).map_err(|(owner, cause)| (Self::Here(owner), cause)),
            Self::Other(owner) => tail(owner).map_err(|(owner, cause)| (Self::Other(owner), cause)),
        }
    }
}
/// Sealed implementation vocabulary for finite typed input custody.
/// Ordinary callers use the pack and selected-operation methods, not these delegates.
pub trait WorthQueryWorkflowInputList<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>:
    sealed::List
{
    type Operation;
    type Completed;
    type Terminal;
    #[doc(hidden)]
    fn registration_denial(
        &self,
        binding: &str,
        target: Option<
            &worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity,
        >,
    ) -> Option<WorthQueryWorkflowInputDenial>;
    #[doc(hidden)]
    fn select<Spec: ApplicationWorkflowSpec<Schema = S>>(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        workflow: WorthQueryWorkflowVocabulary<'_, S, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Self::Operation, WorthQueryWorkflowInputDenial>;
    #[doc(hidden)]
    fn advance(
        operation: Self::Operation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryWorkflowOperationAdvance<Self::Operation, Self::Completed, Self::Terminal>;
    #[doc(hidden)]
    fn required(operation: &Self::Operation) -> &RequiredWorkflowOperation;
    #[doc(hidden)]
    fn completed_required(operation: &Self::Completed) -> &RequiredWorkflowOperation;
    #[doc(hidden)]
    fn close(
        operation: Self::Operation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Operation, WorthQueryOperationCloseDenial)>;
    #[doc(hidden)]
    fn close_completed(
        operation: Self::Completed,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Completed, WorthQueryOperationCloseDenial)>;
    #[doc(hidden)]
    fn close_terminal(
        operation: Self::Terminal,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Terminal, WorthQueryOperationCloseDenial)>;
}
