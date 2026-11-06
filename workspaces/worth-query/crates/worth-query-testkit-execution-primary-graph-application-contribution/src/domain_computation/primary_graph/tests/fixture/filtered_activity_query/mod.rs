use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterRef;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ExactlyOneResult;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

pub struct SelectedActivityParameters;
pub struct SelectedActivityKeyParameter;
pub struct SelectedActivityRelationSlot;
pub struct SelectedActivitySequenceSlot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedActivityResult {
    pub sequence: u64,
}

worth_query_declaration::worth_query_portable_type!(SelectedActivityResult => "worth.query.test.execution.selected_activity.result.v1");
worth_query_declaration::worth_query_portable_type!(SelectedActivityRelationSlot => "worth.query.test.execution.selected_activity.relation.v1");
worth_query_declaration::worth_query_portable_type!(SelectedActivitySequenceSlot => "worth.query.test.execution.selected_activity.sequence.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub SelectedActivityParametersBinding for SelectedActivityParameters { identity: "worth.query.test.execution.selected_activity.parameters.v1" });
worth_query_declaration::worth_query_structured_value_binding!(pub SelectedActivityResultBinding for SelectedActivityResult { identity: "worth.query.test.execution.selected_activity.result.v1" });
worth_query_declaration::worth_query_structured_value_binding!(SelectedActivityUnitBinding for () { identity: "worth.rust.unit" });
worth_query_application_query!(
    pub SelectedActivityQuery for IdentityExecutionSchema,
    identity "worth.query.test.execution.selected_activity.query.v1",
    parameters SelectedActivityParametersBinding,
    result SelectedActivityResultBinding,
    scope Account => "Account",
    name "selected_activity"
);

impl SelectedActivityResult {
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
}

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        SelectedActivityQuery,
    > for SelectedActivityResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            SelectedActivityQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        Ok(Self {
            sequence: row
                .one(selected_activity_relation())?
                .field(selected_activity_sequence())?,
        })
    }
}

pub fn selected_activity_parameters(
    status: &str,
    activity: &str,
) -> ApplicationQueryParameterSet<SelectedActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(status_parameter(), status.to_owned())
        .unwrap()
        .bind(selected_activity_parameter(), activity.to_owned())
        .unwrap()
}

pub fn selected_activity_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    SelectedActivityQuery,
    SelectedActivityParameters,
    SelectedActivityResult,
    Account,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        SelectedActivityQuery,
        Activity,
        (),
        SelectedActivityUnitBinding,
    >::new(Activity::reference())
    .field(selected_activity_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        SelectedActivityQuery,
        Account,
        SelectedActivityResult,
        SelectedActivityResultBinding,
    >::new(Account::reference())
    .relation_where_equal(
        selected_activity_relation(),
        activity,
        ActivityIdentity::at("ActivityIdentity"),
        selected_activity_parameter(),
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(SelectedActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter::<SelectedActivityQuery>())
        .parameter(selected_activity_parameter())
        .where_equal(AccountStatus::at("AccountStatus"), status_parameter())
        .build()
        .unwrap()
}

pub fn selected_activity_parameter() -> ApplicationQueryParameterRef<
    SelectedActivityQuery,
    SelectedActivityKeyParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("activity")
}

pub fn selected_activity_relation() -> ApplicationQueryResultRelationRef<
    SelectedActivityQuery,
    SelectedActivityRelationSlot,
    IdentityExecutionSchema,
    AccountAllActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one("activity", AccountAllActivity::reference())
}

pub fn selected_activity_sequence() -> ApplicationQueryResultFieldRef<
    SelectedActivityQuery,
    SelectedActivitySequenceSlot,
    IdentityExecutionSchema,
    ActivitySequence,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}
