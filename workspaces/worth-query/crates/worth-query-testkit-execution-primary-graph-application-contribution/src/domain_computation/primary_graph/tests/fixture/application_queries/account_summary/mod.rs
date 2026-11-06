//! Account summary query definitions: plain, scoped, governed and ordered.
use super::*;

pub fn account_summary_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    AccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    definition(
        AccountSummaryQuery::reference(),
        ApplicationQueryDisclosureContract::public(),
        false,
        true,
        true,
    )
}

pub fn scoped_account_summary_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ScopedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ScopedAccountSummaryQuery,
        Account,
        AccountSummaryResult,
        ScopedAccountSummaryQueryResultBinding,
    >::new(Account::reference())
    .field(status_result_field::<ScopedAccountSummaryQuery>())
    .field(label_result_field::<ScopedAccountSummaryQuery>())
    .build();
    ApplicationQueryDefinitionBuilder::declare(ScopedAccountSummaryQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .requires_ability(ViewAccount::reference())
        .build()
        .unwrap()
}

pub fn governed_account_summary_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    GovernedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    definition(
        GovernedAccountSummaryQuery::reference(),
        ApplicationQueryDisclosureContract::governed_by(
            "account-holder",
            TouchAccountCapability::reference(),
        )
        .use_field_by(
            AccountStatus::at("AccountStatus"),
            encoded_disclosure(CapabilityDisclosure::AccountActivity),
            ApplicationQueryInfluenceContract::permit_all(),
        )
        .expect("the native capability field is declared")
        .disclose_field_by(
            status_result_field::<GovernedAccountSummaryQuery>(),
            encoded_disclosure(CapabilityDisclosure::AccountActivity),
            ApplicationQueryInfluenceContract::forbid_all(),
        )
        .disclose_field_by(
            label_result_field::<GovernedAccountSummaryQuery>(),
            encoded_disclosure(CapabilityDisclosure::AccountActivity),
            ApplicationQueryInfluenceContract::forbid_all(),
        ),
        false,
        false,
        false,
    )
}

pub fn ordered_account_summary_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    OrderedAccountSummaryQuery,
    AccountSummaryParameters,
    AccountSummaryResult,
    Account,
> {
    definition(
        OrderedAccountSummaryQuery::reference(),
        ApplicationQueryDisclosureContract::public(),
        true,
        false,
        false,
    )
}
