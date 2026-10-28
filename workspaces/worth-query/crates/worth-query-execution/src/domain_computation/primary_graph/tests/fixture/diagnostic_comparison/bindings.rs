//! Ordinary installed bindings retain each actual query and scope meaning.
use super::super::{AccountPolicy, ExternalMapping, IdentityBinding, Principal};
use super::*;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryBindingLimits, ApplicationQueryFieldScope,
    },
    application_schema::{NoApplicationUnit, ReadWrite, U64ApplicationValueBinding},
};

macro_rules! binding {
    ($binding:ident, $query:ty, $identity:literal) => {
        pub struct $binding;
        impl ApplicationQueryBinding<IdentityExecutionSchema> for $binding {
            type Input = AccountSummaryParameters;
            type InputBinding = DiagnosticParametersBinding;
            type Query = $query;
            type ParameterBinding = DiagnosticParametersBinding;
            type ResultBinding = DiagnosticResultBinding;
            type ScopeBinding = ApplicationQueryFieldScope<IdentityExecutionSchema, Account,
                AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit>;
            type PrincipalBinding = IdentityBinding;
            type Mapping = ExternalMapping;
            type Principal = Principal;
            type PrincipalIdentity = u64;
            type PrincipalIdentityBinding = U64ApplicationValueBinding;
            const IDENTITY: &'static str = $identity;
            const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(4);
            fn scope_field() -> worth_query_declaration::facade::application_schema::ApplicationFieldRef<
                IdentityExecutionSchema, Account, AccountPolicy, AccountStatus, String, ReadWrite,
                worth_query_declaration::facade::application_schema::EqualityPredicate, NoApplicationUnit> {
                AccountStatus::reference()
            }
            fn principal_binding() -> worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRef<
                IdentityExecutionSchema, IdentityBinding, ExternalMapping, Principal, u64, U64ApplicationValueBinding> {
                IdentityBinding::reference()
            }
        }
    };
}
binding!(
    DiagnosticAccountBinding,
    DiagnosticAccountQuery,
    "worth.query.test.diagnostic-account.v1"
);
binding!(
    GovernedDiagnosticAccountBinding,
    DiagnosticAccountQuery,
    "worth.query.test.diagnostic-account-governed.v1"
);
binding!(
    DeniedDiagnosticInternalBinding,
    DeniedDiagnosticInternalQuery,
    "worth.query.test.diagnostic-internal-denied.v1"
);
binding!(
    DeniedDiagnosticRelationBinding,
    DeniedDiagnosticRelationQuery,
    "worth.query.test.diagnostic-relation-denied.v1"
);
binding!(
    DeniedDiagnosticFieldBinding,
    DeniedDiagnosticFieldQuery,
    "worth.query.test.diagnostic-field-denied.v1"
);
