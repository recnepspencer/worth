//! One exact completed-chain handoff through the ordinary capability mutation funnel.
use super::progress::{IntentBinding, MutationInput, MutationScope};
use super::{
    PublishedWorkflowInstanceRef, RequiredWorkflowSuccession,
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
        WorthQueryPreserveOutput,
    },
    application_program::ApplicationWorkflowSpec,
};
use worth_query_execution::{
    facade::application_installation::WorthQueryWorkflowVocabulary,
    publication_boundary::workflow_succession::WorthQueryWorkflowSuccessionAdapter,
};
use worth_query_installation::facade::ApplicationSchema;
#[derive(Clone, Debug, Eq, PartialEq)]
/// A refused exact handoff. No native succession slot is consumed on refusal.
pub enum WorthQueryWorkflowSuccessionBindingDenial {
    RuntimeMismatch,
    RequirementMismatch,
    Identity(ApplicationMutationIdentityDenial),
}
impl std::fmt::Display for WorthQueryWorkflowSuccessionBindingDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "workflow succession binding denied: {self:?}")
    }
}
impl std::error::Error for WorthQueryWorkflowSuccessionBindingDenial {}
impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where Schema: ApplicationSchema, Intent: ApplicationMutationIntent<Schema> + Clone,
    IntentBinding<Schema, Intent>: ApplicationCapabilityMutationBinding<Schema>,
    <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema, <IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
    MutationInput<Schema, Intent>: Clone + ApplicationCapabilityRequest<Schema, <IntentBinding<Schema, Intent> as ApplicationCapabilityMutationBinding<Schema>>::Capability, Scope=MutationScope<Schema, IntentBinding<Schema, Intent>>>,
{
    /// Admit the exact body capability and native completed predecessor. Its
    /// supported definition pin is derived from that predecessor, not chosen by a flag.
    pub fn prepare_workflow_succession<Spec, Role>(mut self, workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        predecessor: PublishedWorkflowInstanceRef) -> Result<RequiredWorkflowSuccession, PreparationDenial>
    where Spec: ApplicationWorkflowSpec<Schema=Schema>,
        Role: WorthQueryApplicationOutputRole<Schema=Schema, Contract=<IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Output, Action=WorthQueryPreserveOutput> {
        let workflow = workflow.into();
        let application = self.application_runtime();
        if !std::ptr::eq(application, workflow.runtime()) { return Err(PreparationDenial::RuntimeMismatch); }
        let scope = self.request_scope().clone();
        application.with_application_advancement(&scope, |phase| {
            let selected = application.on_branch(self.product_branch()).select(&phase).map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection).map_err(PreparationDenial::RequestAdmission)?;
            let staged = self.stage().map_err(PreparationDenial::RequestAdmission)?;
            let identities = self.identities().map_err(PreparationDenial::RequestAdmission)?;
            let prepared = authorization::prepare_capability_selected(&phase, &self, &identities, staged, &selected).map_err(PreparationDenial::RequestAdmission)?;
            WorthQueryWorkflowSuccessionAdapter::prepare::<Schema, IntentBinding<Schema, Intent>, Role, Spec>(
                &phase, &selected, workflow.workflow_spec_for(&selected), predecessor, &identities, prepared.admission,
            ).map_err(PreparationDenial::InstancePreparation)
        }).map_err(|cause| PreparationDenial::RequestAdmission(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)))?
    }
    /// Bind exact input/key/role/branch to its one-shot native slot. Recorded
    /// replay resolves before taking that slot or applying new-commit refusals.
    pub fn for_workflow_succession<Spec, Role>(self, workflow: impl Into<WorthQueryWorkflowVocabulary<'application, Schema, Spec>>,
        required: &RequiredWorkflowSuccession) -> Result<Self, WorthQueryWorkflowSuccessionBindingDenial>
    where Spec: ApplicationWorkflowSpec<Schema=Schema>,
        Role: WorthQueryApplicationOutputRole<Schema=Schema, Contract=<IntentBinding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Output, Action=WorthQueryPreserveOutput> {
        let workflow = workflow.into();
        if !std::ptr::eq(self.application_runtime(), workflow.runtime()) { return Err(WorthQueryWorkflowSuccessionBindingDenial::RuntimeMismatch); }
        let input = self.encode_input().map_err(WorthQueryWorkflowSuccessionBindingDenial::Identity)?;
        let identities = self.identities_with_input(&input).map_err(WorthQueryWorkflowSuccessionBindingDenial::Identity)?;
        if !required.matches::<Schema, IntentBinding<Schema, Intent>, Role, Spec>(self.product_branch(), identities.key_identity(), identities.input_identity())
            || self.workflow_transition_identity().is_some() || self.workflow_birth_identity().is_some() || self.workflow_succession_identity().is_some() {
            return Err(WorthQueryWorkflowSuccessionBindingDenial::RequirementMismatch);
        }
        Ok(self.bind_workflow_succession(*required.identity_bytes(), input, required.authority_slot()))
    }
}
