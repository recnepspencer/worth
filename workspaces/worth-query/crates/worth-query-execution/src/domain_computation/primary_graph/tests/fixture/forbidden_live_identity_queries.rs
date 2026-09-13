use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility, ApplicationQueryLiveCauseBinding,
    ApplicationQueryLiveResourceContract, ApplicationQueryObservableInfluence,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef, ApplicationQueryParameterSet,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationScalarValueBinding,
    ApplicationStructuredValueBinding, StringApplicationValueBinding,
};
use worth_query_declaration::worth_query_application_query;

use super::application_queries::AccountSummaryParameters;
use super::live_account_query::{LiveActivityEffect, LiveActivityEvent, LiveActivityEventBinding};
use super::{
    Account, AccountAllActivity, AccountIdentity, AccountPolicy, Activity, ActivityFacts,
    ActivityIdentity, CapabilityDisclosure, CapabilityDisclosureBinding, IdentityExecutionSchema,
    TouchAccountCapability,
};

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct AccountIdentityParameter;
pub struct AccountIdentitySlot;
pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;

worth_query_declaration::worth_query_portable_type!(AccountIdentitySlot => "worth.query.test.execution.forbidden_live.account_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.forbidden_live.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.forbidden_live.activity_identity_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForbiddenLiveIdentityResult;
worth_query_declaration::worth_query_portable_type!(ForbiddenLiveIdentityResult => "worth.query.test.execution.forbidden_live.result.v1");

worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenLiveScopeIdentityQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenLiveScopeIdentityQueryResultBinding for ForbiddenLiveIdentityResult { identity: "worth.query.test.execution.forbidden_live.result.v1" });
worth_query_application_query!(
    pub ForbiddenLiveScopeIdentityQuery for IdentityExecutionSchema,
    identity "ForbiddenLiveScopeIdentityQuery",
    parameters ForbiddenLiveScopeIdentityQueryParametersBinding,
    result ForbiddenLiveScopeIdentityQueryResultBinding,
    scope Account => "Account",
    name "forbidden_live_scope_identity"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenLiveTargetIdentityQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ForbiddenLiveTargetIdentityQueryResultBinding for ForbiddenLiveIdentityResult { identity: "worth.query.test.execution.forbidden_live.result.v1" });
worth_query_application_query!(
    pub ForbiddenLiveTargetIdentityQuery for IdentityExecutionSchema,
    identity "ForbiddenLiveTargetIdentityQuery",
    parameters ForbiddenLiveTargetIdentityQueryParametersBinding,
    result ForbiddenLiveTargetIdentityQueryResultBinding,
    scope Account => "Account",
    name "forbidden_live_target_identity"
);

pub(super) fn forbidden_live_scope_identity_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ForbiddenLiveScopeIdentityQuery,
    AccountSummaryParameters,
    ForbiddenLiveIdentityResult,
    Account,
> {
    definition::<ForbiddenLiveScopeIdentityQuery, ForbiddenLiveScopeIdentityCause>(
        ForbiddenLiveScopeIdentityQuery::reference(),
        influence_without_live_membership(),
        ApplicationQueryInfluenceContract::permit_all(),
    )
}

pub(super) fn forbidden_live_target_identity_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ForbiddenLiveTargetIdentityQuery,
    AccountSummaryParameters,
    ForbiddenLiveIdentityResult,
    Account,
> {
    definition::<ForbiddenLiveTargetIdentityQuery, ForbiddenLiveTargetIdentityCause>(
        ForbiddenLiveTargetIdentityQuery::reference(),
        ApplicationQueryInfluenceContract::permit_all(),
        influence_without_live_membership(),
    )
}

pub(in crate::domain_computation::primary_graph) fn forbidden_live_identity_parameters<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryParameterSet<Query> {
    ApplicationQueryParameterSet::new()
        .bind(account_parameter::<Query>(), "account-1".to_owned())
        .expect("fixture account identity must encode")
}

fn definition<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
            IdentityExecutionSchema,
        > + 'static,
    Cause,
>(
    reference: worth_query_declaration::facade::application_query::ApplicationQueryReference<
        IdentityExecutionSchema,
        Query,
        AccountSummaryParameters,
        ForbiddenLiveIdentityResult,
        Account,
    >,
    scope_influence: ApplicationQueryInfluenceContract,
    target_influence: ApplicationQueryInfluenceContract,
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    Query,
    AccountSummaryParameters,
    ForbiddenLiveIdentityResult,
    Account,
>
where
    Query::ResultBinding: ApplicationStructuredValueBinding<Value = ForbiddenLiveIdentityResult>,
    Cause: ApplicationQueryLiveCauseBinding<
        IdentityExecutionSchema,
        Query,
        Account,
        Activity,
        ScopeIdentityBinding = StringApplicationValueBinding,
        TargetIdentityBinding = StringApplicationValueBinding,
    >,
{
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Query,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity::<Query>());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        Query,
        Account,
        ForbiddenLiveIdentityResult,
        Query::ResultBinding,
    >::new(Account::reference())
    .field(account_identity::<Query>())
    .relation(activities::<Query>(), activity)
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "forbidden-live-identity",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountIdentity::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        scope_influence,
    )
    .use_field_by(
        ActivityIdentity::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        target_influence,
    )
    .disclose_field_by(
        account_identity::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_relation_by(
        activities::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        activity_identity::<Query>(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(reference)
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 2))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .public()
        .parameter(account_parameter::<Query>())
        .where_equal(AccountIdentity::reference(), account_parameter::<Query>())
        .order_by(
            activity_identity::<Query>(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(activities::<Query>())
        .live_by::<Activity, Cause, _, _, _, _, _, _, _, _>(
            account_identity::<Query>(),
            activity_identity::<Query>(),
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .unwrap()
}

fn influence_without_live_membership() -> ApplicationQueryInfluenceContract {
    ApplicationQueryInfluenceContract::permit([
        ApplicationQueryObservableInfluence::RowPresence,
        ApplicationQueryObservableInfluence::Ordering,
        ApplicationQueryObservableInfluence::Pagination,
        ApplicationQueryObservableInfluence::Count,
        ApplicationQueryObservableInfluence::HistoricalMembership,
        ApplicationQueryObservableInfluence::Preview,
    ])
}

pub struct ForbiddenLiveScopeIdentityCause;
pub struct ForbiddenLiveTargetIdentityCause;
worth_query_declaration::worth_query_portable_type!(ForbiddenLiveScopeIdentityCause => "worth.query.test.execution.forbidden_live.scope_cause.v1");
worth_query_declaration::worth_query_portable_type!(ForbiddenLiveTargetIdentityCause => "worth.query.test.execution.forbidden_live.target_cause.v1");

macro_rules! live_cause {
    ($cause:ty, $query:ty) => {
        impl ApplicationQueryLiveCauseBinding<IdentityExecutionSchema, $query, Account, Activity>
            for $cause
        {
            type Effect = LiveActivityEffect;
            type PayloadBinding = LiveActivityEventBinding;
            type ScopeIdentityBinding = StringApplicationValueBinding;
            type TargetIdentityBinding = StringApplicationValueBinding;

            fn effect() -> worth_query_declaration::facade::application_schema::ApplicationEffectRef<
                IdentityExecutionSchema,
                Self::Effect,
                LiveActivityEvent,
            > {
                LiveActivityEffect::reference()
            }

            fn scope_identity(
                payload: &LiveActivityEvent,
            ) -> <Self::ScopeIdentityBinding as ApplicationScalarValueBinding>::Value {
                payload.account().to_owned()
            }

            fn target_identity(
                payload: &LiveActivityEvent,
            ) -> <Self::TargetIdentityBinding as ApplicationScalarValueBinding>::Value {
                payload.activity().to_owned()
            }
        }
    };
}

live_cause!(
    ForbiddenLiveScopeIdentityCause,
    ForbiddenLiveScopeIdentityQuery
);
live_cause!(
    ForbiddenLiveTargetIdentityCause,
    ForbiddenLiveTargetIdentityQuery
);

fn account_parameter<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryParameterRef<Query, AccountIdentityParameter, StringApplicationValueBinding>
{
    ApplicationQueryParameterRef::from_query_identifier("account")
}

fn account_identity<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryResultFieldRef<
    Query,
    AccountIdentitySlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountIdentity,
    String,
    worth_query_declaration::facade::application_schema::ReadOnly,
    worth_query_declaration::facade::application_schema::EqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::reference())
}

fn activity_identity<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryResultFieldRef<
    Query,
    ActivityIdentitySlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivityIdentity,
    String,
    worth_query_declaration::facade::application_schema::ReadOnly,
    worth_query_declaration::facade::application_schema::EqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("activity", ActivityIdentity::reference())
}

fn activities<
    Query: worth_query_declaration::facade::application_query::ApplicationQueryMarkerIdentity<
        IdentityExecutionSchema,
    >,
>() -> ApplicationQueryResultRelationRef<
    Query,
    ActivitiesSlot,
    IdentityExecutionSchema,
    AccountAllActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("activities", AccountAllActivity::reference())
}

fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
