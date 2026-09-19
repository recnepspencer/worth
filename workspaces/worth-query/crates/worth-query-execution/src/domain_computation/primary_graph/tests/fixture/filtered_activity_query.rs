use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
        ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
        ApplicationQueryParameterRef, ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
        ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ExactlyOneResult,
        ForwardResultTraversal,
    },
    application_schema::StringApplicationValueBinding,
};
use worth_query_declaration::worth_query_application_query;

use super::{
    application_queries::status_parameter, Account, AccountAllActivity, AccountStatus, Activity,
    ActivityFacts, ActivityIdentity, ActivitySequence, IdentityExecutionSchema,
};

pub struct SelectedActivityParameters;
pub struct SelectedActivityKeyParameter;
pub struct SelectedActivityRelationSlot;
pub struct SelectedActivitySequenceSlot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedActivityResult {
    sequence: u64,
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
    crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        SelectedActivityQuery,
    > for SelectedActivityResult
{
    fn project(
        row: &crate::domain_computation::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            SelectedActivityQuery,
        >,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial>
    {
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

pub(super) fn selected_activity_definition() -> ApplicationQueryDefinition<
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
        ActivityIdentity::reference(),
        selected_activity_parameter(),
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(SelectedActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 1))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter::<SelectedActivityQuery>())
        .parameter(selected_activity_parameter())
        .where_equal(AccountStatus::reference(), status_parameter())
        .build()
        .unwrap()
}

fn selected_activity_parameter() -> ApplicationQueryParameterRef<
    SelectedActivityQuery,
    SelectedActivityKeyParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("activity")
}

fn selected_activity_relation() -> ApplicationQueryResultRelationRef<
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

fn selected_activity_sequence() -> ApplicationQueryResultFieldRef<
    SelectedActivityQuery,
    SelectedActivitySequenceSlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivitySequence,
    u64,
    worth_query_declaration::facade::application_schema::ReadOnly,
    worth_query_declaration::facade::application_schema::NoEqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::reference())
}
