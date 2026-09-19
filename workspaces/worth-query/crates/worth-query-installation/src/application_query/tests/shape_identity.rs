use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility, ApplicationQueryReference,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::{
    worth_query_application_query, worth_query_portable_type, worth_query_structured_value_binding,
};

use super::{
    installed_schema, Account, AccountActivity, AccountId, Activity, ActivityKind,
    ActivityQueryResult, ActivityQueryResultBinding, ActivitySequence, ActivityStatus,
    QueryTestSchema,
};

pub(super) struct GroupedOneParameters;
pub(super) struct GroupedTwoParameters;
struct FirstKindSlot;
struct FirstSequenceSlot;
struct SecondSequenceSlot;
struct SecondStatusSlot;
struct AccountIdSlot;
struct FirstRelationSlot;
struct SecondRelationSlot;

worth_query_portable_type!(FirstKindSlot => "worth.query.test.installation.shape.first-kind.v1");
worth_query_portable_type!(FirstSequenceSlot => "worth.query.test.installation.shape.first-sequence.v1");
worth_query_portable_type!(SecondSequenceSlot => "worth.query.test.installation.shape.second-sequence.v1");
worth_query_portable_type!(SecondStatusSlot => "worth.query.test.installation.shape.second-status.v1");
worth_query_portable_type!(AccountIdSlot => "worth.query.test.installation.shape.account-id.v1");
worth_query_portable_type!(FirstRelationSlot => "worth.query.test.installation.shape.first-relation.v1");
worth_query_portable_type!(SecondRelationSlot => "worth.query.test.installation.shape.second-relation.v1");
worth_query_structured_value_binding!(
    pub(super) GroupedOneParametersBinding for GroupedOneParameters { identity: "GroupedOneParameters" }
);
worth_query_structured_value_binding!(
    pub(super) GroupedTwoParametersBinding for GroupedTwoParameters { identity: "GroupedTwoParameters" }
);
worth_query_structured_value_binding!(
    GroupedNestedResultBinding for () { identity: "worth.rust.unit" }
);

worth_query_application_query!(
    pub(super) GroupedOneQuery for QueryTestSchema,
    identity "GroupedOneQuery",
    parameters GroupedOneParametersBinding,
    result ActivityQueryResultBinding,
    scope Account => "Account",
    name "grouped_one"
);
worth_query_application_query!(
    pub(super) GroupedTwoQuery for QueryTestSchema,
    identity "GroupedTwoQuery",
    parameters GroupedTwoParametersBinding,
    result ActivityQueryResultBinding,
    scope Account => "Account",
    name "grouped_two"
);

pub(super) fn grouped_one_definition() -> ApplicationQueryDefinition<
    QueryTestSchema,
    GroupedOneQuery,
    GroupedOneParameters,
    ActivityQueryResult,
    Account,
> {
    grouped_shape_definition(GroupedOneQuery::reference(), true)
}

pub(super) fn grouped_two_definition() -> ApplicationQueryDefinition<
    QueryTestSchema,
    GroupedTwoQuery,
    GroupedTwoParameters,
    ActivityQueryResult,
    Account,
> {
    grouped_shape_definition(GroupedTwoQuery::reference(), false)
}

#[test]
fn installed_identity_preserves_which_nested_branch_owns_each_field() {
    let schema = installed_schema();
    let one_then_two = schema
        .certification_query(GroupedOneQuery::reference())
        .unwrap();
    let two_then_one = schema
        .certification_query(GroupedTwoQuery::reference())
        .unwrap();

    assert_ne!(
        one_then_two.read_graph().digest(),
        two_then_one.read_graph().digest()
    );
}

fn grouped_shape_definition<Query, Parameters>(
    reference: ApplicationQueryReference<
        QueryTestSchema,
        Query,
        Parameters,
        ActivityQueryResult,
        Account,
    >,
    split_after_first: bool,
) -> ApplicationQueryDefinition<QueryTestSchema, Query, Parameters, ActivityQueryResult, Account>
where
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        QueryTestSchema,
        ResultBinding = ActivityQueryResultBinding,
    >,
{
    let first = ApplicationQueryResultShapeBuilder::<
        QueryTestSchema,
        Query,
        Activity,
        (),
        GroupedNestedResultBinding,
    >::new(Activity::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        FirstKindSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("kind", ActivityKind::reference()));
    let first = if split_after_first {
        first
    } else {
        first.field(ApplicationQueryResultFieldRef::<
            Query,
            FirstSequenceSlot,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new("sequence", ActivitySequence::reference()))
    };
    let second = ApplicationQueryResultShapeBuilder::<
        QueryTestSchema,
        Query,
        Activity,
        (),
        GroupedNestedResultBinding,
    >::new(Activity::reference());
    let second = if split_after_first {
        second
            .field(ApplicationQueryResultFieldRef::<
                Query,
                SecondSequenceSlot,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
            >::new("sequence", ActivitySequence::reference()))
            .field(ApplicationQueryResultFieldRef::<
                Query,
                SecondStatusSlot,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
                _,
            >::new("status", ActivityStatus::reference()))
    } else {
        second.field(ApplicationQueryResultFieldRef::<
            Query,
            SecondStatusSlot,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
            _,
        >::new("status", ActivityStatus::reference()))
    };
    let shape = ApplicationQueryResultShapeBuilder::<
        QueryTestSchema,
        Query,
        Account,
        ActivityQueryResult,
        ActivityQueryResultBinding,
    >::new(Account::reference())
    .field(ApplicationQueryResultFieldRef::<
        Query,
        AccountIdSlot,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
        _,
    >::new("account_id", AccountId::reference()))
    .relation(
        ApplicationQueryResultRelationRef::<
            Query,
            FirstRelationSlot,
            _,
            _,
            _,
            _,
            ForwardResultTraversal,
            ManyResults,
        >::forward_many("first", AccountActivity::reference()),
        first,
    )
    .relation(
        ApplicationQueryResultRelationRef::<
            Query,
            SecondRelationSlot,
            _,
            _,
            _,
            _,
            ForwardResultTraversal,
            ManyResults,
        >::forward_many("second", AccountActivity::reference()),
        second,
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 2, 4))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap()
}
