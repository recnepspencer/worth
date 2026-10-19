//! Effect-free birth binding retains the original creator and one native enrollment slot.
use super::*;
use crate::application_entry::{RequiredWorkflowBirth, WorthQueryWorkflowBirthBindingDenial};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationIdentities, WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
    },
    application_program::{ApplicationActionDelivery, ApplicationWorkflowSpec},
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;

/// Original creator meaning bound to its issued birth, before any effect or slot consumption.
#[must_use = "retain the creator and its actual native birth requirement"]
pub struct WorthQueryOwnedWorkflowBirthBinding<
    'w,
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
> {
    description: WorthQueryOperationDescription<S, I>,
    required: RequiredWorkflowBirth,
    runtime:
        &'w worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            S,
        >,
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    WorthQueryOwnedWorkflowBirthBinding<'_, S, I>
{
    /// Returns the original effect-free description and shared one-shot requirement.
    pub fn into_parts(self) -> (WorthQueryOperationDescription<S, I>, RequiredWorkflowBirth) {
        (self.description, self.required)
    }
}
impl<S: ApplicationSchema, I: ApplicationMutationIntent<S>> WorthQueryOperationDescription<S, I> {
    /// Checks the exact frozen input, key, branch, binding, role and workflow without re-encoding input.
    /// Refusal returns the original descriptor and leaves the borrowed birth untouched.
    pub fn for_workflow_birth<'w, Spec, Role>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, Spec>>,
        required: &RequiredWorkflowBirth,
    ) -> Result<
        WorthQueryOwnedWorkflowBirthBinding<'w, S, I>,
        (Self, WorthQueryWorkflowBirthBindingDenial),
    >
    where
        Spec: ApplicationWorkflowSpec<Schema = S>,
        Role: WorthQueryApplicationOutputRole<
            Schema = S,
            Contract = <I::Binding as ApplicationMutationBinding<S>>::Output,
            Action = WorthQueryCreateOutput,
        >,
    {
        let workflow = workflow.into();
        if !workflow
            .runtime()
            .product_runtime()
            .owns_branch_selector(self.branch)
        {
            return Err((self, WorthQueryWorkflowBirthBindingDenial::RuntimeMismatch));
        }
        let matches = match &self.encoded_input {
            Ok(input) => ApplicationMutationIdentities::<S, I::Binding>::with_encoded_input(&self.key, input)
                .map(|identities| required.matches::<S, I::Binding, Role, Spec>(
                    self.branch, identities.key_identity(), identities.input_identity())),
            Err(cause) => Err(worth_query_declaration::facade::application_operation::ApplicationMutationIdentityDenial::Input(cause.clone())),
        };
        match matches {
            Ok(true) => Ok(WorthQueryOwnedWorkflowBirthBinding {
                description: self,
                required: required.clone(),
                runtime: workflow.runtime(),
            }),
            Ok(false) => Err((
                self,
                WorthQueryWorkflowBirthBindingDenial::RequirementMismatch,
            )),
            Err(cause) => Err((self, WorthQueryWorkflowBirthBindingDenial::Identity(cause))),
        }
    }
}
impl<'w, S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>>
    WorthQueryOwnedWorkflowBirthBinding<'w, S, I>
{
    /// Selects the declared no-output delivery owner. Failure preserves both original owners.
    pub fn retain_operation<P, A>(
        self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQuerySourceOnlyOperation<
            S,
            I,
            P,
            A,
            <A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode,
        >,
        (Self, WorthQueryOperationConstructionDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        <A::Delivery as ApplicationActionDelivery<S, I::Binding>>::Mode:
            WorthQuerySourceDeliveryMode,
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
            Ok(mut operation) => {
                operation.admission.workflow =
                    Some(super::source_admission::WorkflowAdmission::Birth(required));
                Ok(operation)
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
