use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryLiveCauseBinding, ApplicationQueryLiveResourceContract,
    ApplicationQueryOrderingDirection, ApplicationQueryParameterRef, ApplicationQueryParameterSet,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_query_declaration::{worth_query_application_query, worth_query_effect};

use super::application_queries::AccountSummaryParameters;
use super::{
    Account, AccountAllActivity, AccountIdentity, AccountPolicy, Activity, ActivityFacts,
    ActivityIdentity, ActivitySequence, IdentityExecutionSchema, ViewAccount,
};

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LiveActivityEvent {
    account: String,
    activity: String,
}
worth_query_declaration::worth_query_portable_type!(
    LiveActivityEvent => "worth.query.test.live-activity-event.v1"
);

impl LiveActivityEvent {
    pub(in crate::domain_computation::primary_graph) fn new(
        account: impl Into<String>,
        activity: impl Into<String>,
    ) -> Self {
        Self {
            account: account.into(),
            activity: activity.into(),
        }
    }

    pub(super) fn account(&self) -> &str {
        &self.account
    }

    pub(super) fn activity(&self) -> &str {
        &self.activity
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub LiveActivityEventBinding for LiveActivityEvent { identity: "worth.query.test.live-activity-event.v1" });
impl worth_query_declaration::facade::application_schema::ApplicationRetainedEffectBinding
    for LiveActivityEventBinding
{
    fn retained_bytes(value: &Self::Value) -> u64 {
        u64::try_from(std::mem::size_of::<Self::Value>())
            .unwrap_or(u64::MAX)
            .saturating_add(u64::try_from(value.account.capacity()).unwrap_or(u64::MAX))
            .saturating_add(u64::try_from(value.activity.capacity()).unwrap_or(u64::MAX))
    }
}

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
    account: String,
    activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(LiveAccountActivityResult => "worth.query.test.execution.live.result.v1");

impl LiveAccountActivityResult {
    pub(in crate::domain_computation::primary_graph) fn account(&self) -> &str {
        &self.account
    }

    pub(in crate::domain_computation::primary_graph) fn activities(&self) -> &[(String, u64)] {
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

pub(in crate::domain_computation::primary_graph) fn live_account_parameter(
) -> ApplicationQueryParameterRef<
    LiveAccountActivityQuery,
    AccountIdentityParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

pub(in crate::domain_computation::primary_graph) fn live_account_parameters(
    account: impl Into<String>,
) -> ApplicationQueryParameterSet<LiveAccountActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(live_account_parameter(), account.into())
        .expect("fixture account identity must encode")
}

pub(super) fn live_account_activity_definition() -> ApplicationQueryDefinition<
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot().with_live())
        .requires_ability(ViewAccount::reference())
        .parameter(live_account_parameter())
        .where_equal(AccountIdentity::reference(), live_account_parameter())
        .order_by(
            activity_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(activities())
        .live_by::<Activity, LiveAccountActivityCause, _, _, _, _, _, _, _, _>(
            account_identity(),
            activity_identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .unwrap()
}

impl
    crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        LiveAccountActivityQuery,
    > for LiveAccountActivityResult
{
    fn project(
        row: &crate::domain_computation::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            LiveAccountActivityQuery,
        >,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial>
    {
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

fn account_identity() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
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

fn activity_identity() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
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

fn activity_sequence() -> ApplicationQueryResultFieldRef<
    LiveAccountActivityQuery,
    ActivitySequenceSlot,
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

fn activities() -> ApplicationQueryResultRelationRef<
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
