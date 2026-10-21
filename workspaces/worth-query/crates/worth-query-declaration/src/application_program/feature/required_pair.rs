//! Required lifecycle meaning is retained independently of runtime registration.
use super::ApplicationFeature;
use crate::application_operation::ApplicationMutationBinding;
use crate::application_schema::{ApplicationOperationMarkerIdentity, ApplicationSchema};
use std::any::TypeId;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationRequiredFeatureRoleKind {
    Initial,
    Preserve,
}
impl ApplicationRequiredFeatureRoleKind {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Preserve => "preserve",
        }
    }
}

/// Descriptive output association, never proof of an installed producer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRequiredFeatureOutput {
    family: &'static str,
    source: &'static str,
    role: &'static str,
    family_type: TypeId,
    source_type: TypeId,
}
impl ApplicationRequiredFeatureOutput {
    pub fn of<Family: 'static, Source: 'static>(
        family: &'static str,
        source: &'static str,
        role: &'static str,
    ) -> Self {
        Self {
            family,
            source,
            role,
            family_type: TypeId::of::<Family>(),
            source_type: TypeId::of::<Source>(),
        }
    }
    pub const fn family(&self) -> &'static str {
        self.family
    }
    pub const fn source(&self) -> &'static str {
        self.source
    }
    pub const fn role(&self) -> &'static str {
        self.role
    }
    pub const fn family_type(&self) -> TypeId {
        self.family_type
    }
    pub const fn source_type(&self) -> TypeId {
        self.source_type
    }
}

/// One required slot. Typed identities are local checks; stable names enter meaning.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationRequiredFeatureRoleDeclaration {
    kind: ApplicationRequiredFeatureRoleKind,
    producer: &'static str,
    readiness: &'static str,
    mutation: &'static str,
    operation: &'static str,
    producer_type: TypeId,
    readiness_type: TypeId,
    mutation_type: TypeId,
    operation_type: TypeId,
    output: ApplicationRequiredFeatureOutput,
}
impl ApplicationRequiredFeatureRoleDeclaration {
    pub fn of<Schema, Mutation, Producer: 'static, Readiness: 'static>(
        kind: ApplicationRequiredFeatureRoleKind,
        producer: &'static str,
        readiness: &'static str,
        output: ApplicationRequiredFeatureOutput,
    ) -> Self
    where
        Schema: ApplicationSchema,
        Mutation: ApplicationMutationBinding<Schema>,
    {
        Self {
            kind,
            producer,
            readiness,
            mutation: Mutation::IDENTITY,
            operation:
                <Mutation::Operation as ApplicationOperationMarkerIdentity<Schema>>::IDENTIFIER,
            producer_type: TypeId::of::<Producer>(),
            readiness_type: TypeId::of::<Readiness>(),
            mutation_type: TypeId::of::<Mutation>(),
            operation_type: TypeId::of::<Mutation::Operation>(),
            output,
        }
    }
    pub const fn kind(&self) -> ApplicationRequiredFeatureRoleKind {
        self.kind
    }
    pub const fn producer(&self) -> &'static str {
        self.producer
    }
    pub const fn readiness(&self) -> &'static str {
        self.readiness
    }
    pub const fn mutation(&self) -> &'static str {
        self.mutation
    }
    pub const fn operation(&self) -> &'static str {
        self.operation
    }
    pub const fn producer_type(&self) -> TypeId {
        self.producer_type
    }
    pub const fn readiness_type(&self) -> TypeId {
        self.readiness_type
    }
    pub const fn mutation_type(&self) -> TypeId {
        self.mutation_type
    }
    pub const fn operation_type(&self) -> TypeId {
        self.operation_type
    }
    pub const fn output(&self) -> &ApplicationRequiredFeatureOutput {
        &self.output
    }
    pub(in crate::application_program) fn canonical_fields(&self) -> [(&'static str, String); 8] {
        [
            ("lifecycle", self.kind.canonical_token().to_owned()),
            ("producer", self.producer.to_owned()),
            ("readiness", self.readiness.to_owned()),
            ("mutation", self.mutation.to_owned()),
            ("operation", self.operation.to_owned()),
            ("family", self.output.family.to_owned()),
            ("source", self.output.source.to_owned()),
            ("role", self.output.role.to_owned()),
        ]
    }
}

/// A required pair supplies its two mutation projections without any Host import.
pub trait ApplicationRequiredFeatureRoles<Schema: ApplicationSchema> {
    type Feature: ApplicationFeature<Schema>;
    type InitialMutation: ApplicationMutationBinding<Schema>;
    type PreserveMutation: ApplicationMutationBinding<Schema>;
    fn roles() -> [ApplicationRequiredFeatureRoleDeclaration; 2];
}
