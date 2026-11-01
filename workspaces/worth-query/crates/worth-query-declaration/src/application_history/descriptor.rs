use std::any::TypeId;

use super::{
    ApplicationHistoryAccessBinding, ApplicationHistoryBinding,
    ApplicationHistoryNavigationBindingDescriptor,
};
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
    group_resources: super::ApplicationHistoryGroupResources,
    operations: Vec<ApplicationMutationBindingDescriptor>,
    navigation: Option<ApplicationHistoryNavigationBindingDescriptor>,
}

impl ApplicationHistoryBindingDescriptor {
    pub(super) fn of<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>() -> Self {
        Self {
            binding: TypeId::of::<H>(),
            identity: H::IDENTITY.to_owned(),
            history_type: H::PORTABLE_TYPE_IDENTITY,
            access: ApplicationHistoryAccessBinding::<S, H>::descriptor(),
            group_resources: H::GROUP_RESOURCES,
            operations: Vec::new(),
            navigation: None,
        }
    }

    pub(super) fn push_operation(&mut self, operation: ApplicationMutationBindingDescriptor) {
        self.operations.push(operation);
    }

    pub(super) fn set_navigation(
        &mut self,
        navigation: ApplicationHistoryNavigationBindingDescriptor,
    ) {
        self.navigation = Some(navigation);
    }

    pub fn navigation(&self) -> Option<&ApplicationHistoryNavigationBindingDescriptor> {
        self.navigation.as_ref()
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
    pub const fn group_resources(&self) -> super::ApplicationHistoryGroupResources {
        self.group_resources
    }
    pub fn operations(&self) -> &[ApplicationMutationBindingDescriptor] {
        &self.operations
    }
}
