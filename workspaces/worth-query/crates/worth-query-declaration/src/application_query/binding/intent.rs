use crate::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryParameterSet, ApplicationQueryReference,
};
use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationFieldMarkerIdentity, ApplicationFieldRef, ApplicationIdentityScalarValueBinding,
    ApplicationPrincipalBindingRef, ApplicationSchema, ApplicationStructuredValueBinding,
    EqualityPredicate, WritePosture,
};

use super::{
    ApplicationQueryBindingDescriptor, ApplicationQueryBindingLimits,
    ApplicationQueryPrincipalBindingContract, ApplicationQueryScopeBinding,
    ApplicationQueryScopeContract,
};

/// Complete declaration-time meaning of one application-owned query binding.
pub trait ApplicationQueryBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
    <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope:
        ApplicationEntityMarkerIdentity<Schema>,
    <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Aspect:
        ApplicationAspectMarkerIdentity<
            Schema,
            <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
    <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Field:
        ApplicationFieldMarkerIdentity<
            Schema,
            <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
            <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Aspect,
            Value = <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Value,
        >,
{
    type Input: 'static;
    type InputBinding: ApplicationStructuredValueBinding<Value = Self::Input>;
    type Query: ApplicationQueryMarkerIdentity<
            Schema,
            ParameterBinding = Self::ParameterBinding,
            ResultBinding = Self::ResultBinding,
            Scope = <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        > + 'static;
    type ParameterBinding: ApplicationStructuredValueBinding;
    type ResultBinding: ApplicationStructuredValueBinding;
    type ScopeBinding: ApplicationQueryScopeBinding<Schema>;
    type PrincipalBinding: 'static;
    type Mapping: 'static;
    type Principal: 'static;
    type PrincipalIdentity: 'static;
    type PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<
        Value = Self::PrincipalIdentity,
    >;

    const IDENTITY: &'static str;
    const LIMITS: ApplicationQueryBindingLimits;

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Aspect,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Field,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Value,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Write,
        EqualityPredicate,
        <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Unit,
    >;

    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Schema,
        Self::PrincipalBinding,
        Self::Mapping,
        Self::Principal,
        Self::PrincipalIdentity,
        Self::PrincipalIdentityBinding,
    >;

    #[doc(hidden)]
    fn descriptor() -> ApplicationQueryBindingDescriptor {
        let reference = ApplicationQueryReference::<
            Schema,
            Self::Query,
            <Self::ParameterBinding as ApplicationStructuredValueBinding>::Value,
            <Self::ResultBinding as ApplicationStructuredValueBinding>::Value,
            <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >::from_declaration();
        let field = Self::scope_field();
        ApplicationQueryBindingDescriptor::new::<Self::InputBinding, Self::Query>(
            Self::IDENTITY,
            reference.name(),
            reference.query_type(),
            reference.parameter_type(),
            reference.result_type(),
            reference.scope_type(),
            ApplicationQueryScopeContract::new(
                Self::ScopeBinding::RESOLUTION_MODE,
                field.binding_recipe(),
                <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Write::WRITABLE,
            ),
            ApplicationQueryPrincipalBindingContract::from_reference(Self::principal_binding()),
            Self::LIMITS,
        )
    }
}

/// Request value whose construction and scope lowering are owned by a binding.
pub trait ApplicationQueryIntent<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Binding: ApplicationQueryBinding<Schema, Input = Self>;

    fn reference(
        &self,
    ) -> ApplicationQueryReference<
        Schema,
        <Self::Binding as ApplicationQueryBinding<Schema>>::Query,
        <<Self::Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
        <<Self::Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        <<Self::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >{
        ApplicationQueryReference::from_declaration()
    }

    fn parameters(
        &self,
    ) -> ApplicationQueryParameterSet<<Self::Binding as ApplicationQueryBinding<Schema>>::Query>;

    fn into_scope(self) -> <Self::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding;
}
