//! Source group disposal retains the original live or genuine terminal owner.
use super::*;

enum SourceCustody<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    Operation(
        crate::application_entry::operation::WorthQueryClosingSourceOperation<
            S,
            I,
            P,
            A,
            ApplicationSourceCompletion,
        >,
    ),
    Terminal(T<S, I, P, A>),
}

pub struct SourceGroupDisposal<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
> {
    original: SourceCustody<S, I, P, A>,
}

impl<
        S: ApplicationSchema + 'static,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
    > SourceGroupDisposal<S, I, P, A>
where
    A: ApplicationAction<S, I::Binding>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub(super) fn from_operation(
        original: crate::application_entry::operation::WorthQueryClosingSourceOperation<
            S,
            I,
            P,
            A,
            ApplicationSourceCompletion,
        >,
    ) -> Self {
        Self {
            original: SourceCustody::Operation(original),
        }
    }
    pub(super) fn from_terminal(original: T<S, I, P, A>) -> Self {
        Self {
            original: SourceCustody::Terminal(original),
        }
    }
    pub(super) fn target(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        match &self.original {
            SourceCustody::Operation(closing) => closing.target(),
            SourceCustody::Terminal(terminal) => terminal.operation.history_target(),
        }
    }
    pub(super) fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        match self.original {
            SourceCustody::Operation(closing) => closing
                .close(request)
                .map_err(|(closing, cause)| (Self::from_operation(closing), cause)),
            SourceCustody::Terminal(mut terminal) => match terminal.close_for_history(request) {
                Ok(closed) => Ok(closed),
                Err(cause) => Err((Self::from_terminal(terminal), cause)),
            },
        }
    }
}
