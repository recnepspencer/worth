use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryLiveCauseBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterRef;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::facade::application_query::ManyResults;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_application_query;
use worth_query_declaration::worth_query_effect;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::ViewAccount;

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveActivityEvent {
    pub account: String,
    pub activity: String,
}
worth_query_declaration::worth_query_portable_type!(
    LiveActivityEvent => "worth.query.test.live-activity-event.v1"
);

impl LiveActivityEvent {
    pub fn new(account: impl Into<String>, activity: impl Into<String>) -> Self {
        Self {
            account: account.into(),
            activity: activity.into(),
        }
    }

    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn activity(&self) -> &str {
        &self.activity
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub LiveActivityEventBinding for LiveActivityEvent { identity: "worth.query.test.live-activity-event.v1" });

worth_query_effect!(
    pub LiveActivityEffect for IdentityExecutionSchema, payload LiveActivityEventBinding
);

pub struct AccountIdentitySlot;
pub struct AccountIdentityParameter;
pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;
pub struct ActivitySequenceSlot;
worth_query_declaration::worth_query_portable_type!(AccountIdentitySlot => "worth.query.test.execution.live.account_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.live.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.live.activity_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.execution.live.activity_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveAccountActivityResult {
    pub account: String,
    pub activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(LiveAccountActivityResult => "worth.query.test.execution.live.result.v1");

impl LiveAccountActivityResult {
    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn activities(&self) -> &[(String, u64)] {
        &self.activities
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub LiveAccountActivityQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub LiveAccountActivityQueryResultBinding for LiveAccountActivityResult { identity: "worth.query.test.execution.live.result.v1" });
worth_query_application_query!(
    pub LiveAccountActivityQuery for IdentityExecutionSchema,
    identity "LiveAccountActivityQuery",
    parameters LiveAccountActivityQueryParametersBinding,
    result LiveAccountActivityQueryResultBinding,
    scope Account => "Account",
    name "live_account_activity"
);

pub fn live_account_parameter() -> ApplicationQueryParameterRef<
    LiveAccountActivityQuery,
    AccountIdentityParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

pub fn live_account_parameters(
    account: impl Into<String>,
) -> ApplicationQueryParameterSet<LiveAccountActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(live_account_parameter(), account.into())
        .expect("fixture account identity must encode")
}

pub fn live_account_activity_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    LiveAccountActivityQuery,
    AccountSummaryParameters,
    LiveAccountActivityResult,
    Account,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(activity_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        Account,
        LiveAccountActivityResult,
        LiveAccountActivityQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .relation(activities(), activity)
    .build();
    ApplicationQueryDefinitionBuilder::declare(LiveAccountActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .requires_ability(ViewAccount::reference())
        .parameter(live_account_parameter())
        .where_equal(
            AccountIdentity::at("AccountIdentity"),
            live_account_parameter(),
        )
        .order_by(
            activity_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(activities())
        .live_by::<Activity, LiveAccountActivityCause>(account_identity(), activity_identity())
        .build()
        .unwrap()
}

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
    > for LiveAccountActivityResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            LiveAccountActivityQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        let account = row.field(account_identity())?;
        let activities = row
            .many(activities())?
            .iter()
            .map(|activity| {
                Ok((
                    activity.field(activity_identity())?,
                    activity.field(activity_sequence())?,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            account,
            activities,
        })
    }
}

pub struct LiveAccountActivityCause;
worth_query_declaration::worth_query_portable_type!(LiveAccountActivityCause => "worth.query.test.execution.live.cause.v1");

impl
    ApplicationQueryLiveCauseBinding<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
        Account,
        Activity,
    > for LiveAccountActivityCause
{
    type Effect = LiveActivityEffect;
    type PayloadBinding = LiveActivityEventBinding;
    type ScopeIdentityBinding =
        worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
    type TargetIdentityBinding =
        worth_query_declaration::facade::application_schema::StringApplicationValueBinding;

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
        payload.account.clone()
    }

    fn target_identity(
        payload: &LiveActivityEvent,
    ) -> <Self::TargetIdentityBinding as ApplicationScalarValueBinding>::Value {
        payload.activity.clone()
    }
}

pub fn account_identity() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
    AccountIdentitySlot,
    IdentityExecutionSchema,
    AccountIdentity,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

pub fn activity_identity() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
    ActivityIdentitySlot,
    IdentityExecutionSchema,
    ActivityIdentity,
> {
    ApplicationQueryResultFieldRef::new("activity", ActivityIdentity::at("ActivityIdentity"))
}

pub fn activity_sequence() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
    ActivitySequenceSlot,
    IdentityExecutionSchema,
    ActivitySequence,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}

pub fn activities() -> ApplicationQueryResultRelationRef<
    LiveAccountActivityQuery,
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
