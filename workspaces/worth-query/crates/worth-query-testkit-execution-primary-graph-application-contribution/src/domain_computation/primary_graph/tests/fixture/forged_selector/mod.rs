use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ExactlyOneResult;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct ForgedActivitySlot;
pub struct ForgedSequenceSlot;
worth_query_declaration::worth_query_portable_type!(ForgedActivitySlot => "worth.query.test.execution.forged_selector.activity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ForgedSequenceSlot => "worth.query.test.execution.forged_selector.sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForgedSelectorResult;
worth_query_declaration::worth_query_portable_type!(ForgedSelectorResult => "worth.query.test.execution.forged_selector.result.v1");

worth_query_declaration::worth_query_structured_value_binding!(pub ForgedSelectorQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForgedSelectorQueryResultBinding for ForgedSelectorResult { identity: "worth.query.test.execution.forged_selector.result.v1" });
worth_query_application_query!(
    pub ForgedSelectorQuery for IdentityExecutionSchema,
    identity "ForgedSelectorQuery",
    parameters ForgedSelectorQueryParametersBinding,
    result ForgedSelectorQueryResultBinding,
    scope Account => "Account",
    name "forged_selector"
);

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        ForgedSelectorQuery,
    > for ForgedSelectorResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            ForgedSelectorQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        let activity = row.one(activity())?;
        let _: u64 = activity.field(forged_sequence())?;
        Ok(Self)
    }
}

pub fn forged_selector_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ForgedSelectorQuery,
    AccountSummaryParameters,
    ForgedSelectorResult,
    Account,
> {
    let nested = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ForgedSelectorQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(declared_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ForgedSelectorQuery,
        Account,
        ForgedSelectorResult,
        ForgedSelectorQueryResultBinding,
    >::new(Account::reference())
    .relation(activity(), nested)
    .build();
    ApplicationQueryDefinitionBuilder::declare(ForgedSelectorQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter())
        .where_equal(AccountStatus::at("AccountStatus"), status_parameter())
        .build()
        .unwrap()
}

pub fn declared_sequence() -> SequenceSelector {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}

pub fn forged_sequence() -> SequenceSelector {
    ApplicationQueryResultFieldRef::new("invented_output", ActivitySequence::at("ActivitySequence"))
}

pub fn activity() -> ApplicationQueryResultRelationRef<
    ForgedSelectorQuery,
    ForgedActivitySlot,
    IdentityExecutionSchema,
    AccountPrimaryActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one(
        "primary_activity",
        AccountPrimaryActivity::reference(),
    )
}

pub type SequenceSelector = ApplicationQueryResultFieldRef<
    ForgedSelectorQuery,
    ForgedSequenceSlot,
    IdentityExecutionSchema,
    ActivitySequence,
>;
