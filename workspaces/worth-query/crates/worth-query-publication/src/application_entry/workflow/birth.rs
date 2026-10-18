//! A creator's one mutation funnel, with native enrollment authority attached.
use super::progress::{IntentBinding, MutationInput, MutationScope};
use super::{
    PublishedWorkflowDefinitionRef, RequiredWorkflowBirth,
    WorthQueryWorkflowInstancePreparationDenial as PreparationDenial,
};
use crate::application_entry::{
    mutation::{authorization, WorthQueryApplicationMutationRequestWithIdempotency},
    WorthQueryApplicationRequestMutationDenial,
};
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationBinding,
        ApplicationMutationIdentityDenial, ApplicationMutationIntent,
        ApplicationMutationScopeResolution, WorthQueryApplicationOutputRole,
        WorthQueryCreateOutput,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::facade::application_installation::WorthQueryWorkflowVocabulary;
use worth_query_execution::publication_boundary::workflow_birth::WorthQueryWorkflowBirthAdapter;
use worth_query_installation::facade::ApplicationSchema;

#[derive(Clone, Debug, Eq, PartialEq)]
/// Why an existing keyed request cannot bind to a prepared native birth.
pub enum WorthQueryWorkflowBirthBindingDenial {
    /// The workflow vocabulary belongs to another application runtime.
    RuntimeMismatch,
    /// The binding, role, workflow, branch, logical key or exact input differs.
    RequirementMismatch,
    /// The input or logical key could not be canonically encoded.
    Identity(ApplicationMutationIdentityDenial),
}
impl std::fmt::Display for WorthQueryWorkflowBirthBindingDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workflow birth binding denied: {self:?}")
    }
}
impl std::error::Error for WorthQueryWorkflowBirthBindingDenial {}

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema, <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + ApplicationCapabilityRequest<Schema, <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability, Scope = MutationScope<Schema, IntentBinding<Schema, Intent>>>,
{
    /// Freshly authorizes the exact creator input and the current definition.
    /// Nothing is written until the same request executes its installed body.
    pub fn prepare_workflow_birth<Spec, Role>(
        mut self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        definition: PublishedWorkflowDefinitionRef,
    ) -> Result<RequiredWorkflowBirth, PreparationDenial>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Output, Action = WorthQueryCreateOutput>,
    {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) { return Err(PreparationDenial::RuntimeMismatch); }
        let scope = self.request_scope().clone();
        application.with_application_advancement(&scope, |phase| {
            let selected = application.on_branch(self.product_branch()).select(&phase).map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection).map_err(PreparationDenial::RequestAdmission)?;
            let staged = self.stage().map_err(PreparationDenial::RequestAdmission)?;
            let identities = self.identities().map_err(PreparationDenial::RequestAdmission)?;
            let prepared = authorization::prepare_capability_selected(&phase, &self, &identities, staged, &selected).map_err(PreparationDenial::RequestAdmission)?;
            WorthQueryWorkflowBirthAdapter::prepare::<Schema, IntentBinding<Schema, Intent>, Role, Spec>(
                &phase, &selected, workflow.workflow_spec_for(&selected), definition, &identities, prepared.admission,
            ).map_err(PreparationDenial::InstancePreparation)
        }).map_err(|cause| PreparationDenial::RequestAdmission(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }

    /// An exact recorded outcome resolves before this shared one-shot slot is
    /// taken. A changed input, key, role, definition request or branch cannot
    /// re-use the prepared birth.
    pub fn for_workflow_birth<Spec, Role>(
        self,
        workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        required: &RequiredWorkflowBirth,
    ) -> Result<Self, WorthQueryWorkflowBirthBindingDenial>
    where Spec: ApplicationWorkflowSpec<Schema = Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Output, Action = WorthQueryCreateOutput>,
    {
        let workflow = workflow.into();
        if !std::ptr::eq(self.application_runtime(), workflow.runtime()) { return Err(WorthQueryWorkflowBirthBindingDenial::RuntimeMismatch); }
        let input = self.encode_input().map_err(WorthQueryWorkflowBirthBindingDenial::Identity)?;
        let identities = self.identities_with_input(&input).map_err(WorthQueryWorkflowBirthBindingDenial::Identity)?;
        if !required.matches::<Schema, IntentBinding<Schema, Intent>, Role, Spec>(self.product_branch(), identities.key_identity(), identities.input_identity())
            || self.workflow_transition_identity().is_some() || self.workflow_birth_identity().is_some() || self.workflow_succession_identity().is_some() {
            return Err(WorthQueryWorkflowBirthBindingDenial::RequirementMismatch);
        }
        Ok(self.bind_workflow_birth(*required.identity_bytes(), input, required.authority_slot()))
    }
}
