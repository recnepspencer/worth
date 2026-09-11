use std::marker::PhantomData;

use super::ApplicationRelationIntegrity;

use worth_foundational::facade::{AspectContractRevision, AspectIdentity};

use crate::portable_identity::WorthQueryPortableTypeIdentity;

use super::{
    ApplicationAspectMarkerIdentity, ApplicationEffectMarkerIdentity,
    ApplicationOperationMarkerIdentity, ApplicationStructuredValueBinding,
};

macro_rules! named_reference {
    ($name:ident, $($marker:ident),+) => {
        pub struct $name<$($marker),+> {
            name: &'static str,
            _marker: PhantomData<fn() -> ($($marker),+)>,
        }

        impl<$($marker),+> $name<$($marker),+> {
            #[doc(hidden)]
            pub const fn from_schema_identifier(name: &'static str) -> Self {
                Self {
                    name,
                    _marker: PhantomData,
                }
            }

            pub const fn name(&self) -> &'static str {
                self.name
            }
        }

        impl<$($marker),+> Copy for $name<$($marker),+> {}

        impl<$($marker),+> Clone for $name<$($marker),+> {
            fn clone(&self) -> Self {
                *self
            }
        }

        impl<$($marker),+> std::fmt::Debug for $name<$($marker),+> {
            fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter
                    .debug_struct(stringify!($name))
                    .field("name", &self.name)
                    .finish_non_exhaustive()
            }
        }

        impl<$($marker),+> PartialEq for $name<$($marker),+> {
            fn eq(&self, other: &Self) -> bool {
                self.name == other.name
            }
        }

        impl<$($marker),+> Eq for $name<$($marker),+> {}
    };
}

named_reference!(ApplicationEntityRef, Schema, Entity);
named_reference!(ApplicationAspectRef, Schema, Entity, Aspect);
named_reference!(ApplicationPolicyRef, Schema, Policy);
named_reference!(ApplicationUnitRef, Schema, Unit);

impl<Schema, Entity, Aspect> ApplicationAspectRef<Schema, Entity, Aspect>
where
    Aspect: ApplicationAspectMarkerIdentity<Schema, Entity>,
{
    pub const fn identity(&self) -> AspectIdentity {
        Aspect::ASPECT_IDENTITY
    }

    pub const fn revision(&self) -> AspectContractRevision {
        Aspect::CONTRACT_REVISION
    }
}

pub struct ApplicationAbilityRef<Schema, Ability, Scope> {
    name: &'static str,
    scope: &'static str,
    _marker: PhantomData<fn() -> (Schema, Ability, Scope)>,
}

impl<Schema, Ability, Scope> Copy for ApplicationAbilityRef<Schema, Ability, Scope> {}

impl<Schema, Ability, Scope> Clone for ApplicationAbilityRef<Schema, Ability, Scope> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Ability, Scope> ApplicationAbilityRef<Schema, Ability, Scope> {
    #[doc(hidden)]
    pub const fn from_schema_identifiers(name: &'static str, scope: &'static str) -> Self {
        Self {
            name,
            scope,
            _marker: PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn scope(&self) -> &'static str {
        self.scope
    }
}

impl<Schema, Ability, Scope> std::fmt::Debug for ApplicationAbilityRef<Schema, Ability, Scope> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationAbilityRef")
            .field("name", &self.name)
            .field("scope", &self.scope)
            .finish_non_exhaustive()
    }
}

impl<Schema, Ability, Scope> PartialEq for ApplicationAbilityRef<Schema, Ability, Scope> {
    fn eq(&self, other: &Self) -> bool {
        (self.name, self.scope) == (other.name, other.scope)
    }
}

impl<Schema, Ability, Scope> Eq for ApplicationAbilityRef<Schema, Ability, Scope> {}

pub struct ApplicationRelationRef<Schema, Relation, From, To> {
    name: &'static str,
    from: &'static str,
    to: &'static str,
    integrity: ApplicationRelationIntegrity,
    _marker: PhantomData<fn() -> (Schema, Relation, From, To)>,
}

impl<Schema, Relation, From, To> Copy for ApplicationRelationRef<Schema, Relation, From, To> {}

impl<Schema, Relation, From, To> Clone for ApplicationRelationRef<Schema, Relation, From, To> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Relation, From, To> std::fmt::Debug
    for ApplicationRelationRef<Schema, Relation, From, To>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationRelationRef")
            .field("name", &self.name)
            .field("from", &self.from)
            .field("to", &self.to)
            .field("integrity", &self.integrity)
            .finish_non_exhaustive()
    }
}

impl<Schema, Relation, From, To> PartialEq for ApplicationRelationRef<Schema, Relation, From, To> {
    fn eq(&self, other: &Self) -> bool {
        (self.name, self.from, self.to, self.integrity)
            == (other.name, other.from, other.to, other.integrity)
    }
}

impl<Schema, Relation, From, To> Eq for ApplicationRelationRef<Schema, Relation, From, To> {}

impl<Schema, Relation, From, To> ApplicationRelationRef<Schema, Relation, From, To> {
    #[doc(hidden)]
    pub const fn from_schema_identifiers(
        name: &'static str,
        from: &'static str,
        to: &'static str,
        integrity: ApplicationRelationIntegrity,
    ) -> Self {
        Self {
            name,
            from,
            to,
            integrity,
            _marker: PhantomData,
        }
    }

    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn from(&self) -> &'static str {
        self.from
    }

    pub const fn to(&self) -> &'static str {
        self.to
    }

    pub const fn integrity(&self) -> ApplicationRelationIntegrity {
        self.integrity
    }
}

pub struct ApplicationOperationRef<Schema, Operation, Input> {
    name: &'static str,
    input_identity: &'static str,
    _membership: ApplicationOperationMembership<Schema, Operation, Input>,
    _marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

struct ApplicationOperationMembership<Schema, Operation, Input>(
    PhantomData<fn(Input) -> (Schema, Operation)>,
);

impl<Schema, Operation, Input> Copy for ApplicationOperationMembership<Schema, Operation, Input> {}

impl<Schema, Operation, Input> Clone for ApplicationOperationMembership<Schema, Operation, Input> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Operation, Input> Clone for ApplicationOperationRef<Schema, Operation, Input> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Operation, Input> std::fmt::Debug
    for ApplicationOperationRef<Schema, Operation, Input>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationOperationRef")
            .field("name", &self.name)
            .field("input_identity", &self.input_identity)
            .finish_non_exhaustive()
    }
}

impl<Schema, Operation, Input> PartialEq for ApplicationOperationRef<Schema, Operation, Input> {
    fn eq(&self, other: &Self) -> bool {
        (self.name, &self.input_identity) == (other.name, &other.input_identity)
    }
}

impl<Schema, Operation, Input> Eq for ApplicationOperationRef<Schema, Operation, Input> {}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    #[doc(hidden)]
    pub const fn from_declaration() -> Self {
        Self {
            name: Operation::IDENTIFIER,
            input_identity: Operation::InputBinding::IDENTITY_NAME,
            _membership: ApplicationOperationMembership(PhantomData),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn input_identity(&self) -> WorthQueryPortableTypeIdentity {
        WorthQueryPortableTypeIdentity::declared(self.input_identity)
    }
}

pub struct ApplicationEffectRef<Schema, Effect, Payload> {
    name: &'static str,
    payload_identity: &'static str,
    _membership: ApplicationEffectMembership<Schema, Effect, Payload>,
    _marker: PhantomData<fn(Payload) -> (Schema, Effect)>,
}

struct ApplicationEffectMembership<Schema, Effect, Payload>(
    PhantomData<fn(Payload) -> (Schema, Effect)>,
);

impl<Schema, Effect, Payload> Copy for ApplicationEffectMembership<Schema, Effect, Payload> {}

impl<Schema, Effect, Payload> Clone for ApplicationEffectMembership<Schema, Effect, Payload> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Effect, Payload> Clone for ApplicationEffectRef<Schema, Effect, Payload> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Schema, Effect, Payload> std::fmt::Debug for ApplicationEffectRef<Schema, Effect, Payload> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ApplicationEffectRef")
            .field("name", &self.name)
            .field("payload_identity", &self.payload_identity)
            .finish_non_exhaustive()
    }
}

impl<Schema, Effect, Payload> PartialEq for ApplicationEffectRef<Schema, Effect, Payload> {
    fn eq(&self, other: &Self) -> bool {
        (self.name, &self.payload_identity) == (other.name, &other.payload_identity)
    }
}

impl<Schema, Effect, Payload> Eq for ApplicationEffectRef<Schema, Effect, Payload> {}

impl<Schema, Effect, Payload> ApplicationEffectRef<Schema, Effect, Payload>
where
    Effect: ApplicationEffectMarkerIdentity<Schema>,
    Effect::PayloadBinding: ApplicationStructuredValueBinding<Value = Payload>,
{
    #[doc(hidden)]
    pub const fn from_declaration() -> Self {
        Self {
            name: Effect::IDENTIFIER,
            payload_identity: Effect::PayloadBinding::IDENTITY_NAME,
            _membership: ApplicationEffectMembership(PhantomData),
            _marker: PhantomData,
        }
    }
}

impl<Schema, Effect, Payload> ApplicationEffectRef<Schema, Effect, Payload> {
    pub const fn name(&self) -> &'static str {
        self.name
    }

    pub const fn payload_identity(&self) -> WorthQueryPortableTypeIdentity {
        WorthQueryPortableTypeIdentity::declared(self.payload_identity)
    }
}
impl<Schema, Operation, Input> Copy for ApplicationOperationRef<Schema, Operation, Input> {}

impl<Schema, Effect, Payload> Copy for ApplicationEffectRef<Schema, Effect, Payload> {}
