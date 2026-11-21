//! Issue a succession slot only from exact fresh capability and native completed-chain facts.
use worth_query_execution_attempt::facade::ports::AdmittedOperationExecutionPort as _;
pub struct WorthQueryWorkflowSuccessionAdapter;

impl WorthQueryWorkflowSuccessionAdapter {
    pub const fn bind_idempotency(
        idempotency: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
        identity: &[u8; 32],
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        idempotency.bind_workflow_instance(identity)
    }
}

use super::super::WorkflowInstancePreparationDenial;
use super::{
    mismatch, ApplicationMutationBinding, ApplicationSchema, ApplicationWorkflowSpec,
    RequiredWorkflowSuccession, TypeId, WorkflowSuccessionAuthority,
    WorkflowSuccessionAuthoritySlot, WorthQueryApplicationOutputRole,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
};
use std::sync::{Arc, Mutex};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationIdentities, ApplicationMutationScopeBinding, WorthQueryPreserveOutput,
    },
    application_schema::ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec;
impl WorthQueryWorkflowSuccessionAdapter {
    pub fn prepare<Schema, Binding, Role, Spec>(
        phase: &WorthQueryAdvancementPhase<'_>,
        selected: &dyn crate::ports::SelectedProductOperationRef<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        predecessor: super::WorkflowSuccessionTarget<Schema>,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        admission: WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    ) -> Result<RequiredWorkflowSuccession, WorkflowInstancePreparationDenial>
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Operation: ApplicationOperationMarkerIdentity<Schema>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = WorthQueryPreserveOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let selected = selected.selected_inner();

        let denied = |subject| WorkflowInstancePreparationDenial::Attempt(mismatch(subject));
        let capability = installed
            .succession_capability_identity_bytes::<Binding, Role>()
            .copied()
            .ok_or_else(|| denied("succession binding is not installed"))?;
        if admission.installed_capability_identity() != Some(capability)
            || admission.governed_input_identity() != Some(identities.input_identity())
        {
            return Err(denied(
                "succession requires its exact fresh capability and input",
            ));
        }
        let execution = phase
            .execution_request_for(&selected.application().runtime.product_runtime)
            .map_err(|_| denied("foreign succession phase"))?;
        let read = selected.begin_instance_read_set(&execution, installed, admission)?;
        let support = selected
            .application()
            .installed_program_support()
            .ok_or_else(|| denied("installed program support unavailable"))?;
        read.into_required_workflow_succession::<Binding, Role, Spec>(
            &execution,
            installed,
            predecessor,
            identities,
            support,
            &selected.application().runtime.authorization_clock,
        )
        .map_err(WorkflowInstancePreparationDenial::Attempt)
    }
}
