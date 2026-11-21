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
pub use binding::{
    guard as guard_birth, WorkflowBirthBindingProof,
};

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

#[doc(hidden)]
pub struct WorkflowBirthAuthorityAttemptParts {
    pub branch: crate::basis::WorthQueryProductBranch,
    pub scope: WorthQueryOperationScopeBinding,
    pub binding: &'static str,
    pub role: &'static str,
    pub input_identity: [u8; 32],
    pub key_identity: [u8; 32],
    pub identity: [u8; 32],
    pub instance_identity: String,
    pub capability: [u8; 32],
    pub facts: Vec<WorthQueryApplicationObservedFact>,
    pub compiled: CompiledWorkflowDefinition,
    pub carry: WorkflowLineageCarry,
    pub refusal: Option<WorthQueryApplicationAttemptDenial>,
    pub subject: fn(&super::super::effect_program::output_correspondence::WorthQueryApplicationOutputCorrespondenceCandidate) -> Result<worth_relational::facade::transactions::CreatedEntityRef, WorthQueryApplicationAttemptDenial>,
}

impl WorkflowBirthAuthority {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorkflowBirthAuthorityAttemptParts,
    ) -> Self {
        Self {
            branch: parts.branch,
            scope: parts.scope,
            binding: parts.binding,
            role: parts.role,
            input_identity: parts.input_identity,
            key_identity: parts.key_identity,
            identity: parts.identity,
            instance_identity: parts.instance_identity,
            capability: parts.capability,
            facts: parts.facts,
            compiled: parts.compiled,
            carry: parts.carry,
            refusal: parts.refusal,
            subject: parts.subject,
        }
    }
}

#[doc(hidden)]
pub struct RequiredWorkflowBirthAttemptParts {
    pub branch: crate::basis::WorthQueryProductBranch,
    pub binding_type: TypeId,
    pub role_type: TypeId,
    pub spec_type: TypeId,
    pub binding: &'static str,
    pub input_identity: [u8; 32],
    pub key_identity: [u8; 32],
    pub identity: [u8; 32],
    pub instance_identity: String,
    pub scope: WorthQueryOperationScopeBinding,
    pub compiled: CompiledWorkflowDefinition,
    pub identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub authority: Arc<WorkflowBirthAuthoritySlot>,
    pub subject_in_receipt: fn(
        &super::super::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        worth_relational::facade::identity::EntityId,
        WorthQueryApplicationAttemptDenial,
    >,
}

impl RequiredWorkflowBirth {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: RequiredWorkflowBirthAttemptParts,
    ) -> Self {
        Self {
            branch: parts.branch,
            binding_type: parts.binding_type,
            role_type: parts.role_type,
            spec_type: parts.spec_type,
            binding: parts.binding,
            input_identity: parts.input_identity,
            key_identity: parts.key_identity,
            identity: parts.identity,
            instance_identity: parts.instance_identity,
            scope: parts.scope,
            compiled: parts.compiled,
            identity_locator: parts.identity_locator,
            authority: parts.authority,
            subject_in_receipt: parts.subject_in_receipt,
        }
    }
}

#[doc(hidden)]
pub struct WorkflowBirthAuthoritySlotAttemptParts {
    pub authority: Mutex<Option<WorkflowBirthAuthority>>,
}

impl WorkflowBirthAuthoritySlot {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorkflowBirthAuthoritySlotAttemptParts,
    ) -> Self {
        Self {
            authority: parts.authority,
        }
    }
}
