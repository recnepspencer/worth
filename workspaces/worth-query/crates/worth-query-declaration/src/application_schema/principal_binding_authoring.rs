use std::marker::PhantomData;

use crate::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
};

use super::{
    ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe, ApplicationFieldRef,
    ApplicationFieldUnit, ApplicationIdentityScalarValueBinding, ApplicationRelationRef,
    EqualityPosture, EqualityPredicate, ReadOnly, ReadWrite,
};

pub struct ApplicationPrincipalBindingRequirements<
    Schema,
    Mapping,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
> {
    pub mapping_identity: ApplicationPrincipalMappingIdentityRequirement<Schema, Mapping>,
    pub mapping_status: ApplicationPrincipalMappingStatusRequirement<Schema, Mapping>,
    pub target: ApplicationPrincipalTargetRequirement<Schema, Mapping, Principal>,
    pub principal_identity: ApplicationPrincipalIdentityRequirement<
        Schema,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >,
}

pub struct ApplicationPrincipalMappingIdentityRequirement<Schema, Mapping> {
    pub(super) entity: &'static str,
    pub(super) aspect: &'static str,
    pub(super) field: &'static str,
    _marker: PhantomData<fn() -> (Schema, Mapping)>,
}

impl<Schema, Mapping> ApplicationPrincipalMappingIdentityRequirement<Schema, Mapping> {
    #[doc(hidden)]
    pub fn from_field<Aspect, Field, Unit>(
        field: ApplicationFieldRef<
            Schema,
            Mapping,
            Aspect,
            Field,
            WorthQueryExternalPrincipalIdentity,
            ReadOnly,
            EqualityPredicate,
            Unit,
        >,
    ) -> Self
    where
        Unit: ApplicationFieldUnit,
    {
        Self {
            entity: field.entity(),
            aspect: field.aspect(),
            field: field.field(),
            _marker: PhantomData,
        }
    }
}

pub struct ApplicationPrincipalMappingStatusRequirement<Schema, Mapping> {
    pub(super) aspect: &'static str,
    pub(super) field: &'static str,
    _marker: PhantomData<fn() -> (Schema, Mapping)>,
}

impl<Schema, Mapping> ApplicationPrincipalMappingStatusRequirement<Schema, Mapping> {
    #[doc(hidden)]
    pub fn from_field<Aspect, Field, Equality, Unit>(
        field: ApplicationFieldRef<
            Schema,
            Mapping,
            Aspect,
            Field,
            WorthQueryPrincipalMappingStatus,
            ReadWrite,
            Equality,
            Unit,
        >,
    ) -> Self
    where
        Equality: EqualityPosture,
        Unit: ApplicationFieldUnit,
    {
        Self {
            aspect: field.aspect(),
            field: field.field(),
            _marker: PhantomData,
        }
    }
}

pub struct ApplicationPrincipalTargetRequirement<Schema, Mapping, Principal> {
    pub(super) relation: &'static str,
    pub(super) principal_entity: &'static str,
    _marker: PhantomData<fn() -> (Schema, Mapping, Principal)>,
}

impl<Schema, Mapping, Principal> ApplicationPrincipalTargetRequirement<Schema, Mapping, Principal> {
    #[doc(hidden)]
    pub fn from_relation<Relation>(
        relation: ApplicationRelationRef<Schema, Relation, Mapping, Principal>,
    ) -> Self {
        Self {
            relation: relation.name(),
            principal_entity: relation.to(),
            _marker: PhantomData,
        }
    }
}

pub struct ApplicationPrincipalIdentityRequirement<
    Schema,
    Principal,
    PrincipalIdentity,
    PrincipalIdentityBinding,
> {
    pub(super) aspect: &'static str,
    pub(super) field: &'static str,
    pub(super) scalar_family: worth_foundational::facade::ScalarAspectType,
    pub(super) value_type: &'static str,
    pub(super) binding_recipe: ApplicationFieldBindingRecipe,
    _marker: PhantomData<
        fn() -> (
            Schema,
            Principal,
            PrincipalIdentity,
            PrincipalIdentityBinding,
        ),
    >,
}

impl<Schema, Principal, PrincipalIdentity, PrincipalIdentityBinding>
    ApplicationPrincipalIdentityRequirement<
        Schema,
        Principal,
        PrincipalIdentity,
        PrincipalIdentityBinding,
    >
{
    #[doc(hidden)]
    pub fn from_field<Aspect, Field, Unit>(
        field: ApplicationFieldRef<
            Schema,
            Principal,
            Aspect,
            Field,
            PrincipalIdentity,
            ReadOnly,
            EqualityPredicate,
            Unit,
        >,
    ) -> Self
    where
        Field: super::DeclaredApplicationFieldValue<
            Value = PrincipalIdentity,
            Binding = PrincipalIdentityBinding,
        >,
        PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<Value = PrincipalIdentity>,
        Unit: ApplicationFieldUnit,
    {
        Self {
            aspect: field.aspect(),
            field: field.field(),
            scalar_family: field.scalar_family(),
            value_type: field.value_type_name(),
            binding_recipe: ApplicationFieldBindingRecipe::of::<PrincipalIdentityBinding>(
                ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            ),
            _marker: PhantomData,
        }
    }
}
