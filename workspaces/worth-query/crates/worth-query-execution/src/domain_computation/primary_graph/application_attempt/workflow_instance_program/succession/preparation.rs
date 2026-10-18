//! Issue a succession slot only from exact fresh capability and native completed-chain facts.
use super::super::{PublishedWorkflowInstanceRef, WorkflowInstancePreparationDenial};
use super::{
    mismatch, ApplicationMutationBinding, ApplicationSchema, ApplicationWorkflowSpec,
    RequiredWorkflowSuccession, TypeId, WorkflowSuccessionAuthority,
    WorkflowSuccessionAuthoritySlot, WorthQueryApplicationOutputRole,
    WorthQueryWorkflowSuccessionAdapter,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQuerySelectedProductOperation,
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
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        installed: &WorthQueryInstalledApplicationWorkflowSpec<Schema, Spec>,
        predecessor: PublishedWorkflowInstanceRef,
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
            .execution_request_for(&selected.application().product_runtime)
            .map_err(|_| denied("foreign succession phase"))?;
        let mut read = selected.begin_instance_read_set(&execution, installed, admission)?;
        let (compiled, carry) = read
            .observe_succession_basis(
                &execution,
                installed,
                &predecessor,
                &selected.application().authorization_clock,
            )
            .map_err(WorkflowInstancePreparationDenial::Attempt)?;
        let (instance_identity, identity) = super::super::intent_identity::succession_identity(
            &compiled,
            predecessor.entity_id(),
            read.admission.scope_entity_id(),
            *identities.key_identity(),
            *identities.input_identity(),
            Binding::IDENTITY,
            Role::NAME,
        )
        .map_err(|()| denied("succession intent identity unavailable"))?;
        let branch = read.lease.product().product_branch();
        let scope = read.admission.operation_scope_binding().clone();
        let authority = WorkflowSuccessionAuthority {
            branch, scope: scope.clone(), predecessor: predecessor.entity_id(), binding: Binding::IDENTITY, role: Role::NAME,
            input_identity: *identities.input_identity(), key_identity: *identities.key_identity(), identity, instance_identity: instance_identity.clone(),
            capability, facts: read.facts.iter().cloned().collect(), compiled: compiled.clone(), carry, refusal: read.new_commit_refusal,
            subject: super::super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::succession_subject::<Schema, Binding, Role>,
        };
        Ok(RequiredWorkflowSuccession {
            branch,
            binding_type: TypeId::of::<Binding>(),
            role_type: TypeId::of::<Role>(),
            spec_type: TypeId::of::<Spec>(),
            binding: Binding::IDENTITY,
            input_identity: *identities.input_identity(),
            key_identity: *identities.key_identity(),
            identity,
            instance_identity,
            scope,
            compiled,
            identity_locator: read.lease.layout.workflow().instance.identity.clone(),
            authority: Arc::new(WorkflowSuccessionAuthoritySlot {
                authority: Mutex::new(Some(authority)),
            }),
            subject_in_receipt: super::publication::subject_in_receipt::<Schema, Binding, Role>,
        })
    }
}
