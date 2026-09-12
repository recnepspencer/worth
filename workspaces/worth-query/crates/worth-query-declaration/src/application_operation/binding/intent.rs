use crate::application_schema::{
    ApplicationAspectMarkerIdentity, ApplicationEntityMarkerIdentity,
    ApplicationFieldMarkerIdentity, ApplicationFieldRef, ApplicationIdentityScalarValueBinding,
    ApplicationOperationMarkerIdentity, ApplicationOperationRef, ApplicationPrincipalBindingRef,
    ApplicationSchema, ApplicationStructuredValueBinding, EqualityPredicate, WritePosture,
};

use super::{
    ApplicationMutationBindingDescriptor, ApplicationMutationOutputContract,
    ApplicationMutationPrincipalBindingContract, ApplicationMutationScopeBinding,
    ApplicationMutationScopeContract,
};
use crate::application_operation::ApplicationCandidateRequirements;

/// Complete declaration-time meaning of one application mutation binding.
pub trait ApplicationMutationBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
    <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope:
        ApplicationEntityMarkerIdentity<Schema>,
    <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Aspect:
        ApplicationAspectMarkerIdentity<
            Schema,
            <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
    <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Field:
        ApplicationFieldMarkerIdentity<
            Schema,
            <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Aspect,
            Value = <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Value,
        >,
{
    type Input: 'static;
    type InputBinding: ApplicationStructuredValueBinding<Value = Self::Input>;
    type Result: 'static;
    type ResultBinding: ApplicationStructuredValueBinding<Value = Self::Result>;
    type IdempotencyKey: 'static;
    type Operation: ApplicationOperationMarkerIdentity<Schema, InputBinding = Self::InputBinding>
        + 'static;
    type Decision: 'static;
    type Denial: 'static;
    type DenialBinding: ApplicationStructuredValueBinding<Value = Self::Denial>;
    type Output: ApplicationMutationOutputContract<Schema>;
    type ScopeBinding: ApplicationMutationScopeBinding<Schema>;
    type PrincipalBinding: 'static;
    type Mapping: 'static;
    type Principal: 'static;
    type PrincipalIdentity: 'static;
    type PrincipalIdentityBinding: ApplicationIdentityScalarValueBinding<
        Value = Self::PrincipalIdentity,
    >;

    const IDENTITY: &'static str;
    const HANDLER_IDENTITY: &'static str;
    const IDEMPOTENCY_IDENTITY: &'static str;
    const CANDIDATES: ApplicationCandidateRequirements;

    fn idempotency_key_identity(key: &Self::IdempotencyKey) -> [u8; 32];

    fn input_identity(input: &Self::Input) -> [u8; 32];

    fn scope_field() -> ApplicationFieldRef<
        Schema,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Aspect,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Field,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Value,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Write,
        EqualityPredicate,
        <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Unit,
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
    fn descriptor() -> ApplicationMutationBindingDescriptor {
        let operation =
            ApplicationOperationRef::<Schema, Self::Operation, Self::Input>::from_declaration();
        let field = Self::scope_field();
        ApplicationMutationBindingDescriptor::new::<
            Self::InputBinding,
            Self::ResultBinding,
            Self::Operation,
            Self::Decision,
            Self::DenialBinding,
            Self::Output,
            Self::IdempotencyKey,
            Schema,
        >(
            Self::IDENTITY,
            operation.name(),
            Self::HANDLER_IDENTITY,
            Self::IDEMPOTENCY_IDENTITY,
            field.entity(),
            ApplicationMutationScopeContract::new(
                Self::ScopeBinding::RESOLUTION_MODE,
                field.binding_recipe(),
                <Self::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Write::WRITABLE,
            ),
            ApplicationMutationPrincipalBindingContract::from_reference(Self::principal_binding()),
            Self::CANDIDATES,
        )
    }
}

/// Domain input whose operation and scope association are owned by an entry binding.
pub trait ApplicationMutationIntent<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Binding: ApplicationMutationBinding<Schema, Input = Self>;

    fn operation(
        &self,
    ) -> ApplicationOperationRef<
        Schema,
        <Self::Binding as ApplicationMutationBinding<Schema>>::Operation,
        Self,
    > {
        ApplicationOperationRef::from_declaration()
    }

    fn scope_binding(&self) -> <Self::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding;
}
