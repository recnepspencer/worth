use std::any::TypeId;

use super::{ApplicationHistoryAccessBinding, ApplicationHistoryBinding};
use crate::application_operation::ApplicationMutationBindingDescriptor;
use crate::application_query::{ApplicationQueryBinding, ApplicationQueryBindingDescriptor};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

/// Compiler-local recipes retained beside portable declaration meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryBindingDescriptor {
    binding: TypeId,
    identity: String,
    history_type: WorthQueryPortableTypeIdentity,
    access: ApplicationQueryBindingDescriptor,
    operations: Vec<ApplicationMutationBindingDescriptor>,
}

impl ApplicationHistoryBindingDescriptor {
    pub(super) fn of<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>() -> Self {
        Self {
            binding: TypeId::of::<H>(),
            identity: H::IDENTITY.to_owned(),
            history_type: H::PORTABLE_TYPE_IDENTITY,
            access: ApplicationHistoryAccessBinding::<S, H>::descriptor(),
            operations: Vec::new(),
        }
    }

    pub(super) fn push_operation(&mut self, operation: ApplicationMutationBindingDescriptor) {
        self.operations.push(operation);
    }

    pub const fn binding_type(&self) -> TypeId {
        self.binding
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn history_type(&self) -> &WorthQueryPortableTypeIdentity {
        &self.history_type
    }
    pub fn access(&self) -> &ApplicationQueryBindingDescriptor {
        &self.access
    }
    pub fn operations(&self) -> &[ApplicationMutationBindingDescriptor] {
        &self.operations
    }
}
