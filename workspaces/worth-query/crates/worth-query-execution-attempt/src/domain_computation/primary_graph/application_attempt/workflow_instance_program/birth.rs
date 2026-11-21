//! Sealed authority for creating a subject and enrolling it in one publication.
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
use crate::domain_computation::primary_graph::workflow::{
    definition::CompiledWorkflowDefinition, instance::WorkflowLineageCarry,
};
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::sync::{Arc, Mutex};
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, WorthQueryApplicationOutputRole,
};
use worth_query_declaration::facade::application_program::ApplicationWorkflowSpec;
use worth_query_installation::facade::ApplicationSchema;

pub mod binding;
pub mod publication;
pub use binding::{guard as guard_birth, WorkflowBirthBindingProof};

/// A freshly authorized enrollment of one exact creator request. Clones share
/// one consumed authority slot; an exact recorded replay needs no fresh slot.
#[derive(Clone)]
pub struct RequiredWorkflowBirth {
    branch: crate::basis::WorthQueryProductBranch,
    binding_type: TypeId,
    role_type: TypeId,
    spec_type: TypeId,
    binding: &'static str,
    input_identity: [u8; 32],
    key_identity: [u8; 32],
    identity: [u8; 32],
    instance_identity: String,
    scope: WorthQueryOperationScopeBinding,
    compiled: CompiledWorkflowDefinition,
    identity_locator: worth_foundational::facade::AspectFieldLocator,
    authority: Arc<WorkflowBirthAuthoritySlot>,
    subject_in_receipt: fn(
        &super::super::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_relational::facade::identity::EntityId,
        WorthQueryApplicationAttemptDenial,
    >,
}

impl std::fmt::Debug for RequiredWorkflowBirth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredWorkflowBirth")
            .field("branch", &self.branch)
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct WorkflowBirthAuthoritySlot {
    authority: Mutex<Option<WorkflowBirthAuthority>>,
}

pub struct WorkflowBirthAuthority {
    branch: crate::basis::WorthQueryProductBranch,
    scope: WorthQueryOperationScopeBinding,
    binding: &'static str,
    role: &'static str,
    input_identity: [u8; 32],
    key_identity: [u8; 32],
    identity: [u8; 32],
    instance_identity: String,
    capability: [u8; 32],
    facts: Vec<WorthQueryApplicationObservedFact>,
    compiled: CompiledWorkflowDefinition,
    carry: WorkflowLineageCarry,
    refusal: Option<WorthQueryApplicationAttemptDenial>,
    subject: fn(&super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate) -> Result<worth_relational::facade::transactions::CreatedEntityRef, WorthQueryApplicationAttemptDenial>,
}

impl std::fmt::Debug for WorkflowBirthAuthority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkflowBirthAuthority")
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}

impl WorkflowBirthAuthoritySlot {
    #[doc(hidden)]
    pub fn take(&self) -> Option<WorkflowBirthAuthority> {
        self.authority
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}

impl RequiredWorkflowBirth {
    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }
    #[doc(hidden)]
    pub fn matches<Schema, Binding, Role, Spec>(
        &self,
        branch: crate::basis::WorthQueryProductBranch,
        key: &[u8; 32],
        input: &[u8; 32],
    ) -> bool
    where
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Role: WorthQueryApplicationOutputRole<Schema = Schema, Contract = Binding::Output>,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        self.binding_type == TypeId::of::<Binding>()
            && self.role_type == TypeId::of::<Role>()
            && self.spec_type == TypeId::of::<Spec>()
            && self.branch == branch
            && &self.key_identity == key
            && &self.input_identity == input
    }
    #[doc(hidden)]
    pub fn authority_slot(&self) -> Arc<WorkflowBirthAuthoritySlot> {
        Arc::clone(&self.authority)
    }
    #[doc(hidden)]
    pub const fn identity_bytes(&self) -> &[u8; 32] {
        &self.identity
    }
}

#[doc(hidden)]

pub fn mismatch(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceAuthorityMismatch,
        subject,
    )
}

impl<Schema, Operation, Input, Scope>
    super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >
where
    Schema: ApplicationSchema,
    Operation:
        worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity<
                Schema,
            > + 'static,
{
    pub fn into_required_workflow_birth<Binding, Role, Spec>(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
            Schema,
            Spec,
        >,
        published: &super::super::PublishedWorkflowDefinitionRef,
        identities: &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<
            '_,
            Schema,
            Binding,
        >,
        clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
    ) -> Result<RequiredWorkflowBirth, WorthQueryApplicationAttemptDenial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = worth_query_declaration::facade::application_operation::WorthQueryCreateOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
    {
        let capability = installed
            .birth_capability_identity_bytes::<Binding, Role>()
            .copied()
            .ok_or_else(|| mismatch("creator enrollment is not installed"))?;
        if self.admission.installed_capability_identity() != Some(capability)
            || self.admission.governed_input_identity() != Some(identities.input_identity())
            || published.branch() != self.lease.product().product_branch()
        {
            return Err(mismatch(
                "birth requires the exact admitted creator capability, input and branch",
            ));
        }
        let (compiled, carry, _) =
            self.observe_workflow_start_basis(execution, installed, published, clock)?;
        let (instance_identity, identity) = super::intent_identity::birth_identity(
            &compiled,
            self.admission.scope_entity_id(),
            *identities.key_identity(),
            *identities.input_identity(),
            Binding::IDENTITY,
            Role::NAME,
        )
        .map_err(|()| mismatch("birth intent identity is unavailable"))?;
        let branch = self.lease.product().product_branch();
        let scope = self.admission.operation_scope_binding().clone();
        let authority = WorkflowBirthAuthority {
            branch,
            scope: scope.clone(),
            binding: Binding::IDENTITY,
            role: Role::NAME,
            input_identity: *identities.input_identity(),
            key_identity: *identities.key_identity(),
            identity,
            instance_identity: instance_identity.clone(),
            capability,
            facts: self.facts.iter().cloned().collect(),
            compiled: compiled.clone(),
            carry,
            refusal: self.new_commit_refusal,
            subject: super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::birth_subject::<Schema, Binding, Role>,
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
            identity_locator: self.lease.layout.workflow().instance.identity.clone(),
            authority: Arc::new(WorkflowBirthAuthoritySlot {
                authority: Mutex::new(Some(authority)),
            }),
            subject_in_receipt: publication::subject_in_receipt::<Schema, Binding, Role>,
        })
    }
}
