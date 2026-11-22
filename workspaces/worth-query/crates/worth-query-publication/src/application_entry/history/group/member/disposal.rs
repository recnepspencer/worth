//! Discharge each real source/result owner before the candidate can be retired.
use super::super::storage::*;
use super::*;
impl<S, I, P, A> Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub(in super::super) fn finish(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        Option<Completion<S, I, P, A>>,
        crate::application_entry::WorthQueryOperationCloseDenial,
    > {
        let member = std::mem::replace(self, Self::Closed);
        match member {
            Self::Completed(Some(completed)) => {
                let current = WorthQueryApplicationRequest {
                    application: request.application,
                    principal: request.principal,
                    scope: request.scope,
                    branch: Mode::<S, I, A>::completed_target(&completed),
                };
                match Mode::<S, I, A>::finish_group_member(completed, &current) {
                    Ok(completion) => Ok(Some(completion)),
                    Err((completed, cause)) => {
                        *self = Self::Completed(Some(completed));
                        Err(cause)
                    }
                }
            }
            Self::Unbound(Some(operation)) => {
                let current = WorthQueryApplicationRequest {
                    application: request.application,
                    principal: request.principal,
                    scope: request.scope,
                    branch: Mode::<S, I, A>::target(&operation),
                };
                match Mode::<S, I, A>::close_operation(operation, &current) {
                    Ok(_) => Ok(None),
                    Err((operation, cause)) => {
                        *self = Self::Unbound(Some(operation));
                        Err(cause)
                    }
                }
            }
            _ => unreachable!("selected or original replay owner"),
        }
    }
    #[inline(never)]
    pub(super) fn dispose(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial> {
        let member = std::mem::replace(self, Self::Closed);
        let result = match member {
            Self::Description(_) => Ok(()),
            Self::Unbound(Some(operation)) | Self::Operation(Some(operation)) => {
                Self::dispose_operation(operation, request, aborting)
            }
            Self::ClosingOriginal(Some(closing)) => {
                Self::close_original(closing, request, aborting)
            }
            Self::Completed(Some(completed)) => Self::dispose_completed(completed, request),
            Self::Terminal(Some(terminal)) => Self::dispose_terminal(terminal, request, aborting),
            _ => unreachable!("each unclosed original owner is retained"),
        };
        result.map_err(|(member, cause)| {
            *self = member;
            cause
        })
    }

    #[inline(never)]
    fn dispose_operation(
        operation: Operation<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<(), (Self, crate::application_entry::WorthQueryOperationCloseDenial)> {
        match Mode::<S, I, A>::begin_group_disposal(operation) {
            Ok(closing) => Self::close_original(closing, request, aborting),
            Err((operation, cause)) => Err((Self::Operation(Some(operation)), cause)),
        }
    }

    #[inline(never)]
    fn dispose_completed(
        completed: Completed<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), (Self, crate::application_entry::WorthQueryOperationCloseDenial)> {
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::completed_target(&completed),
        };
        Mode::<S, I, A>::discard_completed(completed, &current)
            .map_err(|(completed, cause)| (Self::Completed(Some(completed)), cause))
    }

    #[inline(never)]
    fn dispose_terminal(
        terminal: Terminal<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<(), (Self, crate::application_entry::WorthQueryOperationCloseDenial)> {
        match Mode::<S, I, A>::begin_group_terminal_disposal(terminal) {
            Ok(closing) => Self::close_original(closing, request, aborting),
            Err((terminal, cause)) => Err((Self::Terminal(Some(terminal)), cause)),
        }
    }

    #[inline(never)]
    fn close_original(
        closing: Closing<S, I, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<
        (),
        (
            Self,
            crate::application_entry::WorthQueryOperationCloseDenial,
        ),
    > {
        let current = WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: Mode::<S, I, A>::group_disposal_target(&closing),
        };
        Mode::<S, I, A>::close_group_disposal(closing, &current, aborting)
            .map(drop)
            .map_err(|(closing, cause)| (Self::ClosingOriginal(Some(closing)), cause))
    }
}
