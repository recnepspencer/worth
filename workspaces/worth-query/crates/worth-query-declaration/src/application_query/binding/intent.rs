use crate::application_query::{
    ApplicationQueryMarkerIdentity, ApplicationQueryParameterSet, ApplicationQueryReference,
};
use crate::application_schema::fragment::RecordSelection;
use crate::application_schema::{
    ApplicationEntityMarkerIdentity, ApplicationIdentityScalarValueBinding,
    ApplicationPrincipalBindingRef, ApplicationSchema, ApplicationStructuredValueBinding,
    ApplicationValueEncodeDenial, ApplicationValueValidationDenial,
};

use super::{
    ApplicationQueryBindingDescriptor, ApplicationQueryBindingLimits,
    ApplicationQueryPrincipalBindingContract, ApplicationQueryScopeBinding,
    ApplicationQueryScopeContract,
};

type QueryScopeFieldRef<Schema, Binding> = <Binding as ApplicationQueryScopeBinding<Schema>>::Field;

type QueryIntentReference<Schema, Binding> = ApplicationQueryReference<
    Schema,
    <Binding as ApplicationQueryBinding<Schema>>::Query,
    <<Binding as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value,
    <<Binding as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
    <<Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
>;

/// Complete declaration-time meaning of one application-owned query binding.
pub trait ApplicationQueryBinding<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
    <Self::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope:
        ApplicationEntityMarkerIdentity<Schema>,
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

    fn scope_field() -> QueryScopeFieldRef<Schema, Self::ScopeBinding>;

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
                field.expected_recipe(),
            ),
            ApplicationQueryPrincipalBindingContract::from_reference(Self::principal_binding()),
            Self::LIMITS,
        )
    }
}

/// Why an application query request could not be validated or have its scalar
/// parameters encoded before admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationQueryInputPreparationDenial {
    InputValidation(ApplicationValueValidationDenial),
    ParameterEncoding(ApplicationValueEncodeDenial),
}

impl std::fmt::Display for ApplicationQueryInputPreparationDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InputValidation(cause) => write!(formatter, "query input rejected: {cause:?}"),
            Self::ParameterEncoding(cause) => {
                write!(formatter, "query parameter encoding rejected: {cause:?}")
            }
        }
    }
}

impl std::error::Error for ApplicationQueryInputPreparationDenial {}

impl From<ApplicationValueValidationDenial> for ApplicationQueryInputPreparationDenial {
    fn from(cause: ApplicationValueValidationDenial) -> Self {
        Self::InputValidation(cause)
    }
}

impl From<ApplicationValueEncodeDenial> for ApplicationQueryInputPreparationDenial {
    fn from(cause: ApplicationValueEncodeDenial) -> Self {
        Self::ParameterEncoding(cause)
    }
}

/// Request value whose construction and scope lowering are owned by a binding.
pub trait ApplicationQueryIntent<Schema>: Sized + 'static
where
    Schema: ApplicationSchema,
{
    type Binding: ApplicationQueryBinding<Schema, Input = Self>;

    fn reference(&self) -> QueryIntentReference<Schema, Self::Binding> {
        ApplicationQueryReference::from_declaration()
    }

    fn parameters(
        &self,
    ) -> ApplicationQueryParameterSet<<Self::Binding as ApplicationQueryBinding<Schema>>::Query>;

    /// Validates the structured request input (`InputBinding::validate`) and
    /// prepares bound scalar parameters without panicking on validation or
    /// codec refusals.
    fn prepare_parameters(
        &self,
    ) -> Result<
        ApplicationQueryParameterSet<<Self::Binding as ApplicationQueryBinding<Schema>>::Query>,
        ApplicationQueryInputPreparationDenial,
    > {
        <<Self::Binding as ApplicationQueryBinding<Schema>>::InputBinding as ApplicationStructuredValueBinding>::validate(self)?;
        Ok(self.parameters())
    }

    fn into_scope(self) -> <Self::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding;
}

/// A query intent whose installed declaration admits bounded live delivery.
pub trait ApplicationLiveQueryIntent<Schema>: ApplicationQueryIntent<Schema>
where
    Schema: ApplicationSchema,
{
    type Target: 'static;
    type LiveCause: crate::application_query::ApplicationQueryLiveCauseBinding<
        Schema,
        <Self::Binding as ApplicationQueryBinding<Schema>>::Query,
        <<Self::Binding as ApplicationQueryBinding<Schema>>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        Self::Target,
    >;
}
