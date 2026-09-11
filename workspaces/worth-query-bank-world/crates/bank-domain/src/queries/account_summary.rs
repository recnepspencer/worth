use worth_query_decl::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
};
use worth_query_decl::facade::worth_query_application_query;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

use crate::authorization::ViewAccount;
use crate::model::AccountId;
use crate::reads::AccountSummary;
use crate::schema::{Account, BankSchema};

use super::account_summary_projection::{account_summary_shape, project_account_summary};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountSummaryQueryParameters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountSummaryRequest {
    account: AccountId,
}

impl AccountSummaryRequest {
    pub const fn new(account: AccountId) -> Self {
        Self { account }
    }

    pub const fn account(self) -> AccountId {
        self.account
    }
}

pub const fn account_summary(account: AccountId) -> AccountSummaryRequest {
    AccountSummaryRequest::new(account)
}

worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountSummaryQueryParametersBinding for AccountSummaryQueryParameters { identity: "AccountSummaryQueryParameters" });
worth_query_decl::facade::worth_query_structured_value_binding!(pub AccountSummaryQueryResultBinding for AccountSummary { identity: "AccountSummary" });
worth_query_application_query!(
    pub AccountSummaryQuery for BankSchema,
    identity "AccountSummaryQuery",
    parameters AccountSummaryQueryParametersBinding,
    result AccountSummaryQueryResultBinding,
    scope Account => "Account",
    name "account_summary"
);

pub fn account_summary_definition() -> ApplicationQueryDefinition<
    BankSchema,
    AccountSummaryQuery,
    AccountSummaryQueryParameters,
    AccountSummary,
    Account,
> {
    ApplicationQueryDefinitionBuilder::declare(AccountSummaryQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(
            account_summary_shape::<
                AccountSummaryQuery,
                AccountSummary,
                AccountSummaryQueryResultBinding,
            >()
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 6))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccount::reference())
        .build()
        .expect("bank account summary query is statically canonical")
}

impl WorthQueryApplicationProjection<BankSchema, AccountSummaryQuery> for AccountSummary {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, BankSchema, AccountSummaryQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        project_account_summary(row)
    }
}
