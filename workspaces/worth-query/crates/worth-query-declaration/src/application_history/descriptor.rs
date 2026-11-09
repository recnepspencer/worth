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
    navigation_resources: super::ApplicationHistoryNavigationResources,
    retention_resources: super::ApplicationHistoryRetentionResources,
    operations: Vec<ApplicationMutationBindingDescriptor>,
    ownership: Vec<super::ApplicationHistoryOwnershipRelation>,
    navigation: Option<ApplicationHistoryNavigationBindingDescriptor>,
    window: Option<super::ApplicationHistoryRetentionBindingDescriptor>,
    bookmarks: Option<super::ApplicationHistoryRetentionBindingDescriptor>,
}

impl ApplicationHistoryBindingDescriptor {
    pub(super) fn of<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>() -> Self {
        Self {
            binding: TypeId::of::<H>(),
            identity: H::IDENTITY.to_owned(),
            history_type: H::PORTABLE_TYPE_IDENTITY,
            access: ApplicationHistoryAccessBinding::<S, H>::descriptor(),
            group_resources: H::GROUP_RESOURCES,
            navigation_resources: H::NAVIGATION_RESOURCES,
            retention_resources: H::RETENTION_RESOURCES,
            operations: Vec::new(),
            ownership: Vec::new(),
            navigation: None,
            window: None,
            bookmarks: None,
        }
    }

    pub(super) fn push_operation(&mut self, operation: ApplicationMutationBindingDescriptor) {
        self.operations.push(operation);
    }

    pub(super) fn push_ownership(&mut self, relation: super::ApplicationHistoryOwnershipRelation) {
        self.ownership.push(relation);
    }

    pub fn ownership(&self) -> &[super::ApplicationHistoryOwnershipRelation] {
        &self.ownership
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

    pub(super) fn set_window(
        &mut self,
        policy: super::ApplicationHistoryRetentionBindingDescriptor,
    ) {
        self.window = Some(policy);
    }
    pub(super) fn set_bookmarks(
        &mut self,
        policy: super::ApplicationHistoryRetentionBindingDescriptor,
    ) {
        self.bookmarks = Some(policy);
    }
    pub fn window(&self) -> Option<&super::ApplicationHistoryRetentionBindingDescriptor> {
        self.window.as_ref()
    }
    pub fn bookmarks(&self) -> Option<&super::ApplicationHistoryRetentionBindingDescriptor> {
        self.bookmarks.as_ref()
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
    pub const fn navigation_resources(&self) -> super::ApplicationHistoryNavigationResources {
        self.navigation_resources
    }
    pub const fn retention_resources(&self) -> super::ApplicationHistoryRetentionResources {
        self.retention_resources
    }
    pub fn operations(&self) -> &[ApplicationMutationBindingDescriptor] {
        &self.operations
    }
}
