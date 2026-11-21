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
pub use binding::{
    guard as guard_succession, WorkflowSuccessionBindingProof,
};
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

#[doc(hidden)]
pub struct WorkflowSuccessionAuthorityAttemptParts {
    pub branch: crate::basis::WorthQueryProductBranch,
    pub scope: WorthQueryOperationScopeBinding,
    pub predecessor: EntityId,
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
    pub subject: SubjectSelector,
    pub output_match: Option<CompletedOutputExpectation>,
}

impl WorkflowSuccessionAuthority {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorkflowSuccessionAuthorityAttemptParts,
    ) -> Self {
        Self {
            branch: parts.branch,
            scope: parts.scope,
            predecessor: parts.predecessor,
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
            output_match: parts.output_match,
        }
    }
}

#[doc(hidden)]
pub struct RequiredWorkflowSuccessionAttemptParts {
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
    pub authority: Arc<WorkflowSuccessionAuthoritySlot>,
    pub subject_in_receipt: fn(
        &super::super::WorthQueryApplicationCommitReceipt,
    ) -> Result<EntityId, WorthQueryApplicationAttemptDenial>,
}

impl RequiredWorkflowSuccession {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: RequiredWorkflowSuccessionAttemptParts,
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
pub struct WorkflowSuccessionAuthoritySlotAttemptParts {
    pub authority: Mutex<Option<WorkflowSuccessionAuthority>>,
}

impl WorkflowSuccessionAuthoritySlot {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorkflowSuccessionAuthoritySlotAttemptParts,
    ) -> Self {
        Self {
            authority: parts.authority,
        }
    }
}
