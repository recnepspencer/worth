//! Effect-free construction retains original occurrence authority separately from meaning.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;

/// Binds an immutable operation description to its issued workflow requirement.
///
/// Construction verifies the original input, binding, branch and authority slot
/// without executing the effect. `retain_operation` selects the installed action;
/// fresh advancement still performs native admission before using the slot.
#[must_use = "retain the bound original description before effectful advancement"]
pub struct WorthQueryOwnedWorkflowBinding<'w, S: ApplicationSchema, I: ApplicationMutationIntent<S>>
{
    description: WorthQueryOperationDescription<S, I>,
    required: RequiredWorkflowOperation,
    runtime:
        &'w worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            S,
        >,
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    WorthQueryOwnedWorkflowBinding<'_, S, I>
{
    /// Recovers the original effect-free description and issued requirement.
    /// No action has run and no workflow authority slot has been consumed.
    /// The caller keeps its authentic runtime for a later fresh binding attempt.
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryOperationDescription<S, I>,
        RequiredWorkflowOperation,
    ) {
        (self.description, self.required)
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> WorthQueryOperationDescription<S, I> {
    pub fn for_workflow_operation<'w, Spec>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, Spec>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<
        WorthQueryOwnedWorkflowBinding<'w, S, I>,
        (Self, WorthQueryWorkflowOperationBindingDenial),
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
    {
        let workflow = workflow.into();
        if !workflow
            .runtime()
            .product_runtime()
            .owns_branch_selector(self.branch)
        {
            return Err((
                self,
                WorthQueryWorkflowOperationBindingDenial::RuntimeMismatch,
            ));
        }
        let identity = match &self.encoded_input {
            Ok(input) => *input.identity(),
            Err(cause) => {
                let cause = cause.clone();
                return Err((self,WorthQueryWorkflowOperationBindingDenial::Identity(
                worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial::Input(cause))));
            }
        };
        if required.operation()
            != <I::Binding as ApplicationMutationBinding<S>>::Operation::IDENTIFIER
            || required.binding() != Some(I::Binding::IDENTITY)
            || required.workflow_execution_requirements()
                != &I::Binding::workflow_execution_requirements()
            || !I::Binding::REQUIRES_WORKFLOW_AUTHORITY
            || required.input_type()
                != <I::Binding as ApplicationMutationBinding<S>>::InputBinding::IDENTITY.as_str()
            || &identity != required.input_identity()
            || required.branch() != self.branch
        {
            return Err((
                self,
                WorthQueryWorkflowOperationBindingDenial::RequirementMismatch,
            ));
        }
        if !required.authority_slot().was_issued() {
            return Err((
                self,
                WorthQueryWorkflowOperationBindingDenial::AuthorityUnavailable,
            ));
        }
        Ok(WorthQueryOwnedWorkflowBinding {
            description: self,
            required: required.clone(),
            runtime: workflow.runtime(),
        })
    }
}
impl<'w, S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>>
    WorthQueryOwnedWorkflowBinding<'w, S, I>
{
    pub fn retain_operation<P, A>(
        self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryWorkflowOperation<S, I, P, A>,
        (Self, WorthQueryOperationConstructionDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    {
        if !std::ptr::eq(self.runtime, runtime.runtime()) {
            return Err((self, WorthQueryOperationConstructionDenial::ForeignRuntime));
        }
        let Self {
            description,
            required,
            runtime: original_runtime,
        } = self;
        match WorthQuerySourceOnlyOperation::construct(description, runtime) {
            Ok(mut source) => {
                source.admission.workflow = Some(
                    super::super::source_admission::WorkflowAdmission::Operation(required.clone()),
                );
                Ok(WorthQueryWorkflowOperation {
                    source,
                    required,
                    source_completed: false,
                    recovery: super::completion::WorkflowDispatchRecovery::Unopened,
                    recovery_result: None,
                })
            }
            Err((description, cause)) => Err((
                Self {
                    description,
                    required,
                    runtime: original_runtime,
                },
                cause,
            )),
        }
    }
}
