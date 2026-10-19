//! Finite input registration describes installed actions; it does not plan workflow order.
use super::{roster::*, selection::WorthQuerySelectedWorkflowOperation};
use crate::application_entry::{
    WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
    WorthQueryWorkflowOperationBindingDenial,
};
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationIntent},
    application_program::{
        ApplicationAction, ApplicationActionDelivery, ApplicationProgramDefinition,
        ApplicationSourceCompletion, ApplicationWorkflowSpec,
    },
    application_schema::ApplicationSchema,
};
use worth_query_execution::{
    facade::application_installation::{
        WorthQueryProgramApplicationRuntime, WorthQueryWorkflowVocabulary,
    },
    publication_boundary::workflow_advance::RequiredWorkflowOperation,
};

/// Selection/registration refusal without an independently admitted business effect.
/// Registration returns original inputs; failed native binding restores the entry.
#[derive(Debug)]
pub enum WorthQueryWorkflowInputDenial {
    DuplicateBinding,
    NotGuarded,
    NotSupplied,
    AlreadyTransferred,
    Binding(WorthQueryWorkflowOperationBindingDenial),
    Construction(WorthQueryOperationConstructionDenial),
}
impl std::fmt::Display for WorthQueryWorkflowInputDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workflow input selection: {self:?}")
    }
}
impl std::error::Error for WorthQueryWorkflowInputDenial {}

/// Original funded descriptions, at most one per binding in a finite submission.
/// Entries transfer once, only to the existing native owning workflow operation.
/// Registration order never selects the next operation. Repeated operations in
/// a workflow remain legal; a later logical submission supplies their new input.
pub struct WorthQueryWorkflowOperationInputs<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R = WorthQueryWorkflowInputEnd,
> {
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
    roster: R,
}
impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>
    WorthQueryWorkflowOperationInputs<S, P>
{
    pub fn new(runtime: &WorthQueryProgramApplicationRuntime<S, P>) -> Self {
        Self {
            runtime: runtime.clone(),
            roster: WorthQueryWorkflowInputEnd,
        }
    }
}
impl<
        S: ApplicationSchema + 'static,
        P: ApplicationProgramDefinition<S>,
        R: WorthQueryWorkflowInputList<S, P>,
    > WorthQueryWorkflowOperationInputs<S, P, R>
{
    /// Validate actual owning-delivery membership before any submitted action runs.
    /// Refusal returns the original pack and unmodified incoming description.
    pub fn with<A, I>(
        self,
        description: WorthQueryOperationDescription<S, I>,
    ) -> Result<
        WorthQueryWorkflowOperationInputs<S, P, WorthQueryWorkflowInputNode<S, I, A, R>>,
        (
            Self,
            WorthQueryOperationDescription<S, I>,
            WorthQueryWorkflowInputDenial,
        ),
    >
    where
        I: ApplicationMutationIntent<S>,
        A: ApplicationAction<S, I::Binding>,
        A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
        WorthQueryWorkflowInputNode<S, I, A, R>: WorthQueryWorkflowInputList<S, P>,
    {
        let denial = if self.roster.contains(I::Binding::IDENTITY) {
            Some(WorthQueryWorkflowInputDenial::DuplicateBinding)
        } else if !I::Binding::REQUIRES_WORKFLOW_AUTHORITY || I::Binding::WORKFLOW_CONTROL {
            Some(WorthQueryWorkflowInputDenial::NotGuarded)
        } else if !self
            .runtime
            .runtime()
            .product_runtime()
            .owns_branch_selector(description.branch)
        {
            Some(WorthQueryWorkflowInputDenial::Construction(
                WorthQueryOperationConstructionDenial::ForeignRuntime,
            ))
        } else {
            self.runtime
                .installed_program()
                .resolve_action::<I::Binding, A>()
                .err()
                .map(|cause| {
                    WorthQueryWorkflowInputDenial::Construction(
                        WorthQueryOperationConstructionDenial::Action(cause),
                    )
                })
        };
        if let Some(cause) = denial {
            return Err((self, description, cause));
        }
        Ok(WorthQueryWorkflowOperationInputs {
            runtime: self.runtime,
            roster: WorthQueryWorkflowInputNode {
                description: Some(description),
                tail: self.roster,
                action: std::marker::PhantomData,
            },
        })
    }
    /// Binding identity narrows a candidate only. Native binding and construction
    /// still check input, branch, requirements, installed meaning and issued slot.
    pub fn select<'w, Spec: ApplicationWorkflowSpec<Schema = S>>(
        &mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'w, S, Spec>>,
        required: &RequiredWorkflowOperation,
    ) -> Result<WorthQuerySelectedWorkflowOperation<S, P, R>, WorthQueryWorkflowInputDenial> {
        let workflow = workflow.into();
        if !std::ptr::eq(self.runtime.runtime(), workflow.runtime()) {
            return Err(WorthQueryWorkflowInputDenial::Binding(
                WorthQueryWorkflowOperationBindingDenial::RuntimeMismatch,
            ));
        }
        self.roster
            .select(&self.runtime, workflow, required)
            .map(WorthQuerySelectedWorkflowOperation::new)
    }
}
