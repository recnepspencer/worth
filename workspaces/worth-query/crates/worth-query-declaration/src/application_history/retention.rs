//! Current capability recipes for history-window and bookmark changes.
use super::{ApplicationHistoryBinding, ApplicationHistoryModel};
use crate::{
    application_capability::{
        ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRef, ApplicationCapabilityRequest,
    },
    application_schema::{
        ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationSchema,
        ApplicationStructuredValueBinding,
    },
    portable_identity::WorthQueryPortableTypeIdentity,
};
use std::any::TypeId;

/// Current retention-control policy for one history's full model instance.
/// Query resolves the history READ's current principal, then separately admits
/// this installed capability and operation for that same full model instance.
/// The input recipe is pure and carries no historical permission.
pub trait ApplicationHistoryRetentionBinding<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>:
    'static
{
    const IDENTITY: &'static str;
    type Capability: ApplicationCapabilityMarkerIdentity<Schema = S>;
    type Input: ApplicationCapabilityRequest<S, Self::Capability, Scope = ApplicationHistoryModel<S, H>>
        + 'static;
    type InputBinding: ApplicationStructuredValueBinding<Value = Self::Input>;
    type Operation: ApplicationOperationMarkerIdentity<S, InputBinding = Self::InputBinding>
        + 'static;

    fn input(access: &H::Access) -> Self::Input;
}

/// This recipe may be explicitly installed for finite window changes.
pub trait ApplicationHistoryWindowBinding<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>:
    ApplicationHistoryRetentionBinding<S, H>
{
}

/// This recipe may be explicitly installed for bookmark changes.
pub trait ApplicationHistoryBookmarkBinding<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>:
    ApplicationHistoryRetentionBinding<S, H>
{
}

/// Portable reference to an independently admitted retention-control recipe.
/// Its installation position distinguishes window and bookmark authority.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ApplicationHistoryRetentionDeclaration {
    parts: WorthQueryPortableApplicationHistoryRetentionParts,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryPortableApplicationHistoryRetentionParts {
    pub binding: String,
    pub capability: String,
    pub capability_type: WorthQueryPortableTypeIdentity,
    pub operation: String,
    pub input_type: WorthQueryPortableTypeIdentity,
}

impl ApplicationHistoryRetentionDeclaration {
    pub fn from_binding<S, H, N>() -> Self
    where
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryRetentionBinding<S, H>,
    {
        let capability = ApplicationCapabilityRef::<S, N::Capability>::from_declaration();
        let operation = ApplicationOperationRef::<S, N::Operation, N::Input>::from_declaration();
        Self::from_untrusted_parts(WorthQueryPortableApplicationHistoryRetentionParts {
            binding: N::IDENTITY.to_owned(),
            capability: capability.name().to_owned(),
            capability_type: capability.marker_identity(),
            operation: operation.name().to_owned(),
            input_type: operation.input_identity(),
        })
    }

    pub fn from_untrusted_parts(parts: WorthQueryPortableApplicationHistoryRetentionParts) -> Self {
        Self { parts }
    }

    pub fn parts(&self) -> &WorthQueryPortableApplicationHistoryRetentionParts {
        &self.parts
    }

    pub fn canonical_fields(&self) -> [(&'static str, &str); 5] {
        let parts = &self.parts;
        [
            ("binding", &parts.binding),
            ("capability", &parts.capability),
            ("capability-type", parts.capability_type.as_str()),
            ("operation", &parts.operation),
            ("input-type", parts.input_type.as_str()),
        ]
    }

    pub(super) fn is_valid(&self) -> bool {
        self.parts.capability_type.is_valid()
            && self.parts.input_type.is_valid()
            && self.canonical_fields().iter().all(|(_, value)| {
                !value.is_empty()
                    && !value
                        .chars()
                        .any(|ch| ch.is_control() || ch.is_whitespace())
            })
    }
}

/// Compiler-local correspondence for the original typed input/capability recipe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationHistoryRetentionBindingDescriptor {
    binding: TypeId,
    capability: TypeId,
    operation: TypeId,
    input: TypeId,
    input_binding: TypeId,
    declaration: ApplicationHistoryRetentionDeclaration,
}

impl ApplicationHistoryRetentionBindingDescriptor {
    pub fn of<S, H, N>() -> Self
    where
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryRetentionBinding<S, H>,
    {
        Self {
            binding: TypeId::of::<N>(),
            capability: TypeId::of::<N::Capability>(),
            operation: TypeId::of::<N::Operation>(),
            input: TypeId::of::<N::Input>(),
            input_binding: TypeId::of::<N::InputBinding>(),
            declaration: ApplicationHistoryRetentionDeclaration::from_binding::<S, H, N>(),
        }
    }

    pub fn declaration(&self) -> &ApplicationHistoryRetentionDeclaration {
        &self.declaration
    }

    /// Warm correspondence uses declared slots without rebuilding owned text.
    pub fn matches<S, H, N>(&self) -> bool
    where
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryRetentionBinding<S, H>,
    {
        let capability = ApplicationCapabilityRef::<S, N::Capability>::from_declaration();
        let operation = ApplicationOperationRef::<S, N::Operation, N::Input>::from_declaration();
        let parts = self.declaration.parts();
        self.binding == TypeId::of::<N>()
            && self.capability == TypeId::of::<N::Capability>()
            && self.operation == TypeId::of::<N::Operation>()
            && self.input == TypeId::of::<N::Input>()
            && self.input_binding == TypeId::of::<N::InputBinding>()
            && parts.binding == N::IDENTITY
            && parts.capability == capability.name()
            && parts.capability_type == capability.marker_identity()
            && parts.operation == operation.name()
            && parts.input_type == operation.input_identity()
    }
}
