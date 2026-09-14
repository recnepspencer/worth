use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryBindingLimits, ApplicationQueryFieldScope,
};
use worth_query_declaration::facade::application_schema::{
    NoApplicationUnit, ReadWrite, U64ApplicationValueBinding,
};

use super::application_queries::{
    AccountSummaryParameters, AccountSummaryQuery, AccountSummaryQueryParametersBinding,
    AccountSummaryQueryResultBinding,
};
use super::{
    Account, AccountStatus, ExternalMapping, IdentityBinding, IdentityExecutionSchema, Principal,
};

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
        super::AccountPolicy,
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
    const LIMITS: ApplicationQueryBindingLimits = ApplicationQueryBindingLimits::bounded(1, 1);

    fn scope_field() -> worth_query_declaration::facade::application_schema::ApplicationFieldRef<
        IdentityExecutionSchema,
        Account,
        super::AccountPolicy,
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
