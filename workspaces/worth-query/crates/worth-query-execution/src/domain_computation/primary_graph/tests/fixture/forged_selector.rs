use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ExactlyOneResult, ForwardResultTraversal,
};
use worth_query_declaration::worth_query_application_query;

use super::application_queries::{status_parameter, AccountSummaryParameters};
use super::{
    Account, AccountPrimaryActivity, AccountStatus, Activity, ActivityFacts, ActivitySequence,
    IdentityExecutionSchema,
};

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
    crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        ForgedSelectorQuery,
    > for ForgedSelectorResult
{
    fn project(
        row: &crate::domain_computation::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            ForgedSelectorQuery,
        >,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial>
    {
        let activity = row.one(activity())?;
        let _: u64 = activity.field(forged_sequence())?;
        Ok(Self)
    }
}

pub(super) fn forged_selector_definition() -> ApplicationQueryDefinition<
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter())
        .where_equal(AccountStatus::reference(), status_parameter())
        .build()
        .unwrap()
}

fn declared_sequence() -> SequenceSelector {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::reference())
}

fn forged_sequence() -> SequenceSelector {
    ApplicationQueryResultFieldRef::new("invented_output", ActivitySequence::reference())
}

fn activity() -> ApplicationQueryResultRelationRef<
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

type SequenceSelector = ApplicationQueryResultFieldRef<
    ForgedSelectorQuery,
    ForgedSequenceSlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivitySequence,
    u64,
    worth_query_declaration::facade::application_schema::ReadOnly,
    worth_query_declaration::facade::application_schema::NoEqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
>;
