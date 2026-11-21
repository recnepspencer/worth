use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
pub struct WorthQueryWorkflowBirthAdapter;

impl WorthQueryWorkflowBirthAdapter {
    pub const fn bind_idempotency(
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        identity: &[u8; 32],
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_instance(identity)
    }
}

use super::super::{PublishedWorkflowDefinitionRef, WorkflowInstancePreparationDenial};
use super::{
    mismatch, ApplicationMutationBinding, ApplicationSchema, ApplicationWorkflowSpec, Arc, Mutex,
    RequiredWorkflowBirth, TypeId, WorkflowBirthAuthority, WorkflowBirthAuthoritySlot,
    WorthQueryApplicationOutputRole,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationIdentities, ApplicationMutationScopeBinding, WorthQueryCreateOutput,
    },
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;

impl WorthQueryWorkflowBirthAdapter {
    pub fn prepare<Schema, Binding, Role, Spec>(
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &dyn crate::ports::SelectedProductOperationRef<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        published: PublishedWorkflowDefinitionRef,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Result<RequiredWorkflowBirth, WorkflowInstancePreparationDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryCreateOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let selected = selected.selected_inner();

        let denied = |subject| WorkflowInstancePreparationDenial::Attempt(mismatch(subject));
        let capability = installed
            .birth_capability_identity_bytes::<Binding, Role>()
            .copied()
            .ok_or_else(|| denied("creator enrollment is not installed"))?;
        if admission.installed_capability_identity() != Some(capability)
            || admission.governed_input_identity() != Some(identities.input_identity())
            || published.branch() != selected.product().product_branch()
        {
            return Err(denied(
                "birth requires the exact admitted creator capability, input and branch",
            ));
        }
        let execution = phase
            .execution_request_for(&selected.application().runtime.product_runtime)
            .map_err(|_| denied("foreign birth phase"))?;
        let read = selected.begin_instance_read_set(&execution, installed, admission)?;
        read.into_required_workflow_birth::<Binding, Role, Spec>(
            &execution,
            installed,
            &published,
            identities,
            &selected.application().runtime.authorization_clock,
        )
        .map_err(WorkflowInstancePreparationDenial::Attempt)
    }
}
