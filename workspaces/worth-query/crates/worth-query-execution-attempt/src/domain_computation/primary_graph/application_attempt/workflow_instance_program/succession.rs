//! Fresh authority for a completed native chain, not a birth or revived instance.
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationObservedFact,
};
use crate::{
    domain_computation::{
        authorization::WorthQueryOperationScopeBinding,
        primary_graph::workflow::{
            definition::CompiledWorkflowDefinition, instance::WorkflowLineageCarry,
        },
    },
    equality_type_id::EqualityTypeId as TypeId,
};
use std::sync::{Arc, Mutex};
use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, WorthQueryApplicationOutputRole},
    application_program::ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;
pub mod basis;
pub mod binding;
pub mod publication;
pub mod target;
pub use binding::{guard as guard_succession, WorkflowSuccessionBindingProof};
pub use target::WorkflowSuccessionTarget;
type SubjectSelector = fn(&super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate) -> Result<EntityId, WorthQueryApplicationAttemptDenial>;

/// Exact logical request and pin, backed by a shared one-shot native authority slot.
#[derive(Clone)]
pub struct RequiredWorkflowSuccession {
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
    authority: Arc<WorkflowSuccessionAuthoritySlot>,
    subject_in_receipt: fn(
        &super::super::WorthQueryApplicationCommitReceipt,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
}
impl std::fmt::Debug for RequiredWorkflowSuccession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequiredWorkflowSuccession")
            .field("branch", &self.branch)
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}
pub struct WorkflowSuccessionAuthority {
    branch: crate::basis::WorthQueryProductBranch,
    scope: WorthQueryOperationScopeBinding,
    predecessor: EntityId,
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
    subject: SubjectSelector,
    output_match: Option<CompletedOutputExpectation>,
}
pub struct CompletedOutputExpectation {
    declaration: worth_query_declaration::facade::application_schema::ApplicationOperationBodySuccessionOutputMatch,
    target: Result<EntityId, WorthQueryApplicationAttemptDenial>,
}
pub struct WorkflowSuccessionAuthoritySlot {
    authority: Mutex<Option<WorkflowSuccessionAuthority>>,
}
impl WorkflowSuccessionAuthoritySlot {
    #[doc(hidden)]
    pub fn take(&self) -> Option<WorkflowSuccessionAuthority> {
        self.authority
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
    }
}
impl RequiredWorkflowSuccession {
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
    pub fn authority_slot(&self) -> Arc<WorkflowSuccessionAuthoritySlot> {
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
    pub(crate) fn into_required_workflow_succession<Binding, Role, Spec>(
        mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        installed: &worth_query_installation::facade::WorthQueryInstalledApplicationWorkflowSpec<
            Schema,
            Spec,
        >,
        predecessor: WorkflowSuccessionTarget<Schema>,
        identities: &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<
            '_,
            Schema,
            Binding,
        >,
        support: &crate::domain_computation::primary_graph::program_occurrence::WorthQueryInstalledProgramSupport<Schema>,
        clock: &crate::domain_computation::runtime_time::WorthQueryRuntimeClock,
    ) -> Result<RequiredWorkflowSuccession, WorthQueryApplicationAttemptDenial>
    where
        Binding: ApplicationMutationBinding<Schema, Operation = Operation, Input = Input>,
        Role: WorthQueryApplicationOutputRole<
            Schema = Schema,
            Contract = Binding::Output,
            Action = worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput,
        >,
        Spec: ApplicationWorkflowSpec<Schema = Schema>,
{
        let capability = installed
            .succession_capability_identity_bytes::<Binding, Role>()
            .copied()
            .ok_or_else(|| mismatch("succession binding is not installed"))?;
        if self.admission.installed_capability_identity() != Some(capability)
            || self.admission.governed_input_identity() != Some(identities.input_identity())
        {
            return Err(mismatch(
                "succession requires its exact fresh capability and input",
            ));
        }
        let (compiled, carry, output_match, predecessor) =
            self.observe_succession_basis(execution, installed, predecessor, support, clock)?;
        let (instance_identity, identity) = super::intent_identity::succession_identity(
            &compiled,
            predecessor.entity_id(),
            self.admission.scope_entity_id(),
            *identities.key_identity(),
            *identities.input_identity(),
            Binding::IDENTITY,
            Role::NAME,
        )
        .map_err(|()| mismatch("succession intent identity unavailable"))?;
        let branch = self.lease.product().product_branch();
        let scope = self.admission.operation_scope_binding().clone();
        let authority = WorkflowSuccessionAuthority {
            branch,
            scope: scope.clone(),
            predecessor: predecessor.entity_id(),
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
            subject: super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate::succession_subject::<Schema, Binding, Role>,
            output_match,
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
            identity_locator: self.lease.layout.workflow().instance.identity.clone(),
            authority: Arc::new(WorkflowSuccessionAuthoritySlot {
                authority: Mutex::new(Some(authority)),
            }),
            subject_in_receipt: publication::subject_in_receipt::<Schema, Binding, Role>,
        })
    }
}
