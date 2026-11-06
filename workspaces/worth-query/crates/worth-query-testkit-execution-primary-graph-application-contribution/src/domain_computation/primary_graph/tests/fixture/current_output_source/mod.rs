use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBindingLimits;
use worth_query_declaration::facade::application_query::ApplicationQueryFieldScope;
use worth_query_declaration::facade::application_schema::NoApplicationUnit;
use worth_query_declaration::facade::application_schema::ReadWrite;
use worth_query_declaration::facade::application_schema::U64ApplicationValueBinding;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryQuery;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryQueryParametersBinding;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryQueryResultBinding;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::ExternalMapping;
use crate::domain_computation::primary_graph::tests::fixture::IdentityBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;

pub struct TestAccountSourceBinding;

impl ApplicationQueryBinding<IdentityExecutionSchema> for TestAccountSourceBinding {
    type Input = AccountSummaryParameters;
    type InputBinding = AccountSummaryQueryParametersBinding;
    type Query = AccountSummaryQuery;
    type ParameterBinding = AccountSummaryQueryParametersBinding;
    type ResultBinding = AccountSummaryQueryResultBinding;
    type ScopeBinding = ApplicationQueryFieldScope<
        IdentityExecutionSchema,
        Account,
        crate::domain_computation::primary_graph::tests::fixture::AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        NoApplicationUnit,
    >;
    type PrincipalBinding = IdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.test.account-source.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);

    fn scope_field() -> worth_query_declaration::facade::application_schema::ApplicationFieldRef<
        IdentityExecutionSchema,
        Account,
        crate::domain_computation::primary_graph::tests::fixture::AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        worth_query_declaration::facade::application_schema::EqualityPredicate,
        NoApplicationUnit,
    > {
        AccountStatus::reference()
    }

    fn principal_binding(
    ) -> worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRef<
        IdentityExecutionSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        IdentityBinding::reference()
    }
}

/// A distinct installed binding with the same query and parameter meaning.
pub struct AlternateAccountSourceBinding;

impl ApplicationQueryBinding<IdentityExecutionSchema> for AlternateAccountSourceBinding {
    type Input = AccountSummaryParameters;
    type InputBinding = AccountSummaryQueryParametersBinding;
    type Query = AccountSummaryQuery;
    type ParameterBinding = AccountSummaryQueryParametersBinding;
    type ResultBinding = AccountSummaryQueryResultBinding;
    type ScopeBinding = ApplicationQueryFieldScope<
        IdentityExecutionSchema,
        Account,
        crate::domain_computation::primary_graph::tests::fixture::AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        NoApplicationUnit,
    >;
    type PrincipalBinding = IdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;

    const IDENTITY: &'static str = "worth.query.test.alternate-account-source.v1";
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::results(1);

    fn scope_field() -> worth_query_declaration::facade::application_schema::ApplicationFieldRef<
        IdentityExecutionSchema,
        Account,
        crate::domain_computation::primary_graph::tests::fixture::AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        worth_query_declaration::facade::application_schema::EqualityPredicate,
        NoApplicationUnit,
    > {
        AccountStatus::reference()
    }

    fn principal_binding(
    ) -> worth_query_declaration::facade::application_schema::ApplicationPrincipalBindingRef<
        IdentityExecutionSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        IdentityBinding::reference()
    }
}
