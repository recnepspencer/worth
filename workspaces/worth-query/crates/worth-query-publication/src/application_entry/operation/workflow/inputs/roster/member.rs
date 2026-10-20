//! A candidate binding transfers its exact description only after native construction.
use super::*;
use crate::application_entry::{
    WorthQueryCompletedWorkflowOperation, WorthQueryTerminalWorkflowOperation,
    WorthQueryWorkflowOperation,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationAdmissionBinding, ApplicationMutationBinding,
        ApplicationMutationScopeResolution,
    },
    application_program::{
        ApplicationAction, ApplicationActionDelivery, ApplicationSourceCompletion,
    },
};
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>, A, T> sealed::List
    for WorthQueryWorkflowInputNode<S, I, A, T>
{
}
impl<S, I, P, A, T> WorthQueryWorkflowInputList<S, P> for WorthQueryWorkflowInputNode<S, I, A, T>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    T: WorthQueryWorkflowInputList<S, P>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    I::Binding: ApplicationMutationAdmissionBinding<S>,
    <I::Binding as ApplicationMutationAdmissionBinding<S>>::Admission:
        crate::application_entry::mutation::authorization::WorthQueryBindingAdmission<
            S,
            I::Binding,
        >,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeResolution<
        S,
        <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity,
    >,
{
    type Operation =
        WorthQueryWorkflowOperationChoice<WorthQueryWorkflowOperation<S, I, P, A>, T::Operation>;
    type Completed = WorthQueryWorkflowOperationChoice<
        WorthQueryCompletedWorkflowOperation<S, I, P, A>,
        T::Completed,
    >;
    type Terminal = WorthQueryWorkflowOperationChoice<
        WorthQueryTerminalWorkflowOperation<S, I, P, A>,
        T::Terminal,
    >;
    fn capture(
        &self,
        context: &mut super::super::capture::WorthQueryWorkflowInputCapture<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryWorkflowInputDenial> {
        let target = self
            .target
            .as_ref()
            .ok_or(WorthQueryWorkflowInputDenial::UnqualifiedTarget)?;
        let description = self
            .description
            .as_ref()
            .ok_or(WorthQueryWorkflowInputDenial::AlreadyTransferred)?;
        context.retain(target, description)?;
        self.tail.capture(context)
    }
    fn registration_denial(
        &self,
        binding: &str,
        target: Option<
            &worth_query_declaration::facade::application_program::ApplicationWorkflowNodeIdentity,
        >,
    ) -> Option<WorthQueryWorkflowInputDenial> {
        match (self.target.as_ref(), target) {
            (Some(existing), Some(incoming)) if existing == incoming => {
                Some(WorthQueryWorkflowInputDenial::DuplicateTarget)
            }
            (None, _) | (_, None) if binding == I::Binding::IDENTITY => {
                Some(WorthQueryWorkflowInputDenial::DuplicateBinding)
            }
            _ => self.tail.registration_denial(binding, target),
        }
    }
    fn select<Spec: ApplicationWorkflowSpec<Schema = S>>(
        &mut self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        workflow: WorthQueryWorkflowVocabulary<'_, S, Spec>,
        required: &RequiredWorkflowOperation,
    ) -> Result<Self::Operation, WorthQueryWorkflowInputDenial> {
        if required.binding() != Some(I::Binding::IDENTITY)
            || self
                .target
                .as_ref()
                .is_some_and(|target| target.as_str() != required.node_path())
        {
            return self
                .tail
                .select(runtime, workflow, required)
                .map(WorthQueryWorkflowOperationChoice::Other);
        }
        let description = self
            .description
            .take()
            .ok_or(WorthQueryWorkflowInputDenial::AlreadyTransferred)?;
        let bound = match description.for_workflow_operation(workflow, required) {
            Ok(bound) => bound,
            Err((original, cause)) => {
                self.description = Some(original);
                return Err(WorthQueryWorkflowInputDenial::Binding(cause));
            }
        };
        match bound.retain_operation::<P, A>(runtime) {
            Ok(operation) => Ok(WorthQueryWorkflowOperationChoice::Here(operation)),
            Err((bound, cause)) => {
                self.description = Some(bound.into_parts().0);
                Err(WorthQueryWorkflowInputDenial::Construction(cause))
            }
        }
    }
    fn advance(
        operation: Self::Operation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryWorkflowOperationAdvance<Self::Operation, Self::Completed, Self::Terminal> {
        match operation {
            WorthQueryWorkflowOperationChoice::Here(owner) => owner.advance(request).map(
                WorthQueryWorkflowOperationChoice::Here,
                WorthQueryWorkflowOperationChoice::Here,
                WorthQueryWorkflowOperationChoice::Here,
            ),
            WorthQueryWorkflowOperationChoice::Other(owner) => T::advance(owner, request).map(
                WorthQueryWorkflowOperationChoice::Other,
                WorthQueryWorkflowOperationChoice::Other,
                WorthQueryWorkflowOperationChoice::Other,
            ),
        }
    }
    fn required(operation: &Self::Operation) -> &RequiredWorkflowOperation {
        match operation {
            WorthQueryWorkflowOperationChoice::Here(owner) => owner.required(),
            WorthQueryWorkflowOperationChoice::Other(owner) => T::required(owner),
        }
    }
    fn completed_required(operation: &Self::Completed) -> &RequiredWorkflowOperation {
        match operation {
            WorthQueryWorkflowOperationChoice::Here(owner) => owner.required(),
            WorthQueryWorkflowOperationChoice::Other(owner) => T::completed_required(owner),
        }
    }
    fn close(
        operation: Self::Operation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Operation, WorthQueryOperationCloseDenial)> {
        operation.close_with(
            |owner| owner.close(request),
            |owner| T::close(owner, request),
        )
    }
    fn close_completed(
        operation: Self::Completed,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Completed, WorthQueryOperationCloseDenial)> {
        operation.close_with(
            |owner| owner.close(request),
            |owner| T::close_completed(owner, request),
        )
    }
    fn close_terminal(
        operation: Self::Terminal,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self::Terminal, WorthQueryOperationCloseDenial)> {
        operation.close_with(
            |owner| owner.close(request),
            |owner| T::close_terminal(owner, request),
        )
    }
}
