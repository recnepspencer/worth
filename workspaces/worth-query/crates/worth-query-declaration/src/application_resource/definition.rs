use std::marker::PhantomData;

use crate::application_query::{
    ApplicationQueryBindingReference, ApplicationQueryDisclosureContract,
};
use crate::application_schema::ApplicationSchema;
use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationResourceAuthorization, ApplicationResourceBinding, ApplicationResourceLimits,
    ApplicationResourceRequestResponseProfile, ApplicationResourceValueBindingReference,
};

mod validation;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableApplicationResourceParts {
    pub name: String,
    pub resource_type: WorthQueryPortableTypeIdentity,
    pub provider_role: WorthQueryPortableTypeIdentity,
    pub access: ApplicationQueryBindingReference,
    pub input: ApplicationResourceValueBindingReference,
    pub result: ApplicationResourceValueBindingReference,
    pub failure: ApplicationResourceValueBindingReference,
    pub profile: ApplicationResourceRequestResponseProfile,
    pub limits: ApplicationResourceLimits,
    pub authorization: ApplicationResourceAuthorization,
    pub disclosure: ApplicationQueryDisclosureContract,
}

/// Portable meaning, incapable of reconstructing typed input or live custody.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ErasedApplicationResourceDefinition {
    parts: WorthQueryPortableApplicationResourceParts,
}

impl ErasedApplicationResourceDefinition {
    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationResourceParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &WorthQueryPortableApplicationResourceParts {
        &self.parts
    }

    pub fn validate(&self) -> Result<(), ApplicationResourceDefinitionDenial> {
        validation::validate(&self.parts)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApplicationResourceDefinitionDenial {
    InvalidIdentity,
    InvalidAccess,
    UnsupportedValueProfile,
    InvalidRequestResponseProfile,
    InvalidLimits,
    InvalidAuthorization,
    InvalidDisclosure,
}

pub struct ApplicationResourceDefinition<Schema, Binding> {
    erased: ErasedApplicationResourceDefinition,
    _binding: PhantomData<fn() -> (Schema, Binding)>,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>
    ApplicationResourceDefinition<Schema, Binding>
{
    pub fn request_response(
        profile: ApplicationResourceRequestResponseProfile,
        limits: ApplicationResourceLimits,
        authorization: ApplicationResourceAuthorization,
        disclosure: ApplicationQueryDisclosureContract,
    ) -> Result<Self, ApplicationResourceDefinitionDenial> {
        use crate::portable_identity::WorthQueryPortableType;
        let erased = ErasedApplicationResourceDefinition::from_untrusted_parts(
            WorthQueryPortableApplicationResourceParts {
                name: Binding::IDENTITY.to_owned(),
                resource_type: Binding::PORTABLE_TYPE_IDENTITY,
                provider_role: Binding::Provider::PORTABLE_TYPE_IDENTITY,
                access: ApplicationQueryBindingReference::of::<Schema, Binding::Access>(),
                input: ApplicationResourceValueBindingReference::of::<Binding::InputBinding>(),
                result: ApplicationResourceValueBindingReference::of::<Binding::ResultBinding>(),
                failure: ApplicationResourceValueBindingReference::of::<Binding::FailureBinding>(),
                profile,
                limits,
                authorization,
                disclosure,
            },
        );
        erased.validate()?;
        Ok(Self {
            erased,
            _binding: PhantomData,
        })
    }
}

impl<Schema, Binding> ApplicationResourceDefinition<Schema, Binding> {
    pub fn into_erased(self) -> ErasedApplicationResourceDefinition {
        self.erased
    }
}
