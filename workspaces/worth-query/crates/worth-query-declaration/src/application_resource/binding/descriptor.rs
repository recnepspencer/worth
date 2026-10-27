use std::any::TypeId;

use crate::application_query::{ApplicationQueryBinding, ApplicationQueryBindingDescriptor};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::{WorthQueryPortableType, WorthQueryPortableTypeIdentity};

use super::super::{ApplicationResourceBinding, ApplicationResourceValueBindingReference};

/// Compiler-local recipe correspondence. This never enters package records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationResourceBindingDescriptor {
    identity: String,
    resource: WorthQueryPortableTypeIdentity,
    provider: WorthQueryPortableTypeIdentity,
    resource_type: TypeId,
    access_type: TypeId,
    provider_type: TypeId,
    input_type: TypeId,
    result_type: TypeId,
    failure_type: TypeId,
    access: ApplicationQueryBindingDescriptor,
    values: [ApplicationResourceValueBindingReference; 3],
}

impl ApplicationResourceBindingDescriptor {
    pub(crate) fn of<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>(
    ) -> Self {
        Self {
            identity: Binding::IDENTITY.to_owned(),
            resource: Binding::PORTABLE_TYPE_IDENTITY,
            provider: Binding::Provider::PORTABLE_TYPE_IDENTITY,
            resource_type: TypeId::of::<Binding>(),
            access_type: TypeId::of::<Binding::Access>(),
            provider_type: TypeId::of::<Binding::Provider>(),
            input_type: TypeId::of::<Binding::InputBinding>(),
            result_type: TypeId::of::<Binding::ResultBinding>(),
            failure_type: TypeId::of::<Binding::FailureBinding>(),
            access: Binding::Access::descriptor(),
            values: [
                ApplicationResourceValueBindingReference::of::<Binding::InputBinding>(),
                ApplicationResourceValueBindingReference::of::<Binding::ResultBinding>(),
                ApplicationResourceValueBindingReference::of::<Binding::FailureBinding>(),
            ],
        }
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn resource_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.resource
    }
    pub fn provider_identity(&self) -> &WorthQueryPortableTypeIdentity {
        &self.provider
    }
    pub fn access(&self) -> &ApplicationQueryBindingDescriptor {
        &self.access
    }
    pub fn values(&self) -> &[ApplicationResourceValueBindingReference; 3] {
        &self.values
    }

    pub fn matches_definition(
        &self,
        definition: &super::super::ErasedApplicationResourceDefinition,
    ) -> bool {
        let parts = definition.parts();
        self.identity == parts.name
            && self.resource == parts.resource_type
            && self.provider == parts.provider_role
            && parts.access.matches_descriptor(&self.access)
            && self.values
                == [
                    parts.input.clone(),
                    parts.result.clone(),
                    parts.failure.clone(),
                ]
    }

    pub fn matches_binding<
        Schema: ApplicationSchema,
        Binding: ApplicationResourceBinding<Schema>,
    >(
        &self,
    ) -> bool {
        self.identity == Binding::IDENTITY
            && self.resource == Binding::PORTABLE_TYPE_IDENTITY
            && self.provider == Binding::Provider::PORTABLE_TYPE_IDENTITY
            && self.resource_type == TypeId::of::<Binding>()
            && self.access_type == TypeId::of::<Binding::Access>()
            && self.provider_type == TypeId::of::<Binding::Provider>()
            && self.input_type == TypeId::of::<Binding::InputBinding>()
            && self.result_type == TypeId::of::<Binding::ResultBinding>()
            && self.failure_type == TypeId::of::<Binding::FailureBinding>()
    }
}
