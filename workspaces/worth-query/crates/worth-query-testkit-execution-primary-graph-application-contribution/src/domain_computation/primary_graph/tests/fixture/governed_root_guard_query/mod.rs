use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryRootPath;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;

pub struct RootGuardSequenceSlot;
worth_query_declaration::worth_query_portable_type!(RootGuardSequenceSlot => "worth.query.test.execution.root_guard.sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RootGuardResult {
    pub sequence: u64,
}
worth_query_declaration::worth_query_portable_type!(RootGuardResult => "worth.query.test.execution.root_guard.result.v1");

worth_query_declaration::worth_query_structured_value_binding!(pub GovernedRootGuardQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub GovernedRootGuardQueryResultBinding for RootGuardResult { identity: "worth.query.test.execution.root_guard.result.v1" });
worth_query_application_query!(
    pub GovernedRootGuardQuery for IdentityExecutionSchema,
    identity "GovernedRootGuardQuery",
    parameters GovernedRootGuardQueryParametersBinding,
    result GovernedRootGuardQueryResultBinding,
    scope Account => "Account",
    name "governed_root_guard"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenRootGuardQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenRootGuardQueryResultBinding for RootGuardResult { identity: "worth.query.test.execution.root_guard.result.v1" });
worth_query_application_query!(
    pub ForbiddenRootGuardQuery for IdentityExecutionSchema,
    identity "ForbiddenRootGuardQuery",
    parameters ForbiddenRootGuardQueryParametersBinding,
    result ForbiddenRootGuardQueryResultBinding,
    scope Account => "Account",
    name "forbidden_root_guard"
);

pub fn governed_root_guard_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    GovernedRootGuardQuery,
    AccountSummaryParameters,
    RootGuardResult,
    Account,
> {
    definition(
        GovernedRootGuardQuery::reference(),
        ApplicationQueryInfluenceContract::permit_all(),
    )
}

pub fn forbidden_root_guard_definition() -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    ForbiddenRootGuardQuery,
    AccountSummaryParameters,
    RootGuardResult,
    Account,
> {
    definition(
        ForbiddenRootGuardQuery::reference(),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
}

pub fn definition<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
            IdentityExecutionSchema,
        > + 'static,
>(
    reference: worth_query_declaration::facade::application_query::ApplicationQueryReference<
        IdentityExecutionSchema,
        Query,
        AccountSummaryParameters,
        RootGuardResult,
        Account,
    >,
    guard_influence: ApplicationQueryInfluenceContract,
) -> ApplicationQueryDefinitionBuilder<
    IdentityExecutionSchema,
    Query,
    AccountSummaryParameters,
    RootGuardResult,
    Account,
>
where
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = RootGuardResult>,
{
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Query,
        Activity,
        RootGuardResult,
        Query::ResultBinding,
    >::new(Activity::reference())
    .field(sequence::<Query>())
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "root-guard",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountLabel::at("AccountLabel"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        guard_influence,
    )
    .expect("record field contract")
    .disclose_field_by(
        sequence::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Activity::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Account::reference())
                .where_equal(
                    AccountLabel::at("AccountLabel"),
                    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                        "guard-match".to_owned(),
                    )
                    .expect("fixture root guard must encode"),
                )
                .expect("record field contract")
                .forward(AccountAllActivity::reference()),
        )
}

impl<
        Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
            IdentityExecutionSchema,
        >,
    > WorthQueryApplicationProjection<IdentityExecutionSchema, Query> for RootGuardResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, IdentityExecutionSchema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            sequence: row.field(sequence::<Query>())?,
        })
    }
}

pub fn sequence<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryResultFieldRef<
    Query,
    RootGuardSequenceSlot,
    IdentityExecutionSchema,
    ActivitySequence,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}

pub fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
