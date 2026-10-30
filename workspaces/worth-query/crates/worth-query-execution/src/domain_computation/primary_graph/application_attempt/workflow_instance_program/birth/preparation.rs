use super::super::{PublishedWorkflowDefinitionRef, WorkflowInstancePreparationDenial};
use super::{
    mismatch, ApplicationMutationBinding, ApplicationSchema, ApplicationWorkflowSpec, Arc, Mutex,
    RequiredWorkflowBirth, TypeId, WorkflowBirthAuthority, WorkflowBirthAuthoritySlot,
    WorthQueryApplicationOutputRole, WorthQueryWorkflowBirthAdapter,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryAdvancementPhase,
    WorthQuerySelectedProductOperation,
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
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
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
            .execution_request_for(&selected.application().product_runtime)
            .map_err(|_| denied("foreign birth phase"))?;
        let mut read = selected.begin_instance_read_set(&execution, installed, admission)?;
        let (compiled, carry, _) = read
            .observe_workflow_start_basis(
                &execution,
                installed,
                &published,
                &selected.application().authorization_clock,
            )
            .map_err(WorkflowInstancePreparationDenial::Attempt)?;
        let (instance_identity, identity) = super::super::intent_identity::birth_identity(
            &compiled,
            read.admission.scope_entity_id(),
            *identities.key_identity(),
            *identities.input_identity(),
            Binding::IDENTITY,
            Role::NAME,
        )
        .map_err(|()| denied("birth intent identity is unavailable"))?;
        let branch = read.lease.product().product_branch();
        let scope = read.admission.operation_scope_binding().clone();
        let authority = WorkflowBirthAuthority {
            branch, scope: scope.clone(), binding: Binding::IDENTITY, role: Role::NAME,
            input_identity: *identities.input_identity(), key_identity: *identities.key_identity(), identity,
            instance_identity: instance_identity.clone(), capability,
            facts: read.facts.iter().cloned().collect(), compiled: compiled.clone(), carry,
            refusal: read.new_commit_refusal,
            subject: super::super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::birth_subject::<Schema, Binding, Role>,
        };
        Ok(RequiredWorkflowBirth {
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
            authority: Arc::new(WorkflowBirthAuthoritySlot {
                authority: Mutex::new(Some(authority)),
            }),
            subject_in_receipt: super::publication::subject_in_receipt::<Schema, Binding, Role>,
        })
    }
}
