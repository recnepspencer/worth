use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility, ApplicationQueryLiveCauseBinding,
    ApplicationQueryLiveResourceContract, ApplicationQueryOrderingDirection,
    ApplicationQueryParameterRef, ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ForwardResultTraversal,
    ManyResults,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_query_declaration::worth_query_application_query;

use super::application_queries::AccountSummaryParameters;
use super::live_account_query::{LiveActivityEffect, LiveActivityEvent, LiveActivityEventBinding};
use super::{
    Account, AccountAllActivity, AccountIdentity, AccountLabel, AccountPolicy, Activity,
    ActivityFacts, ActivityIdentity, ActivitySequence, CapabilityDisclosure,
    CapabilityDisclosureBinding, IdentityExecutionSchema, TouchAccountCapability,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct AccountIdentitySlot;
pub struct AccountLabelSlot;
pub struct AccountIdentityParameter;
pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;
pub struct ActivitySequenceSlot;
worth_query_declaration::worth_query_portable_type!(AccountIdentitySlot => "worth.query.test.execution.governed_live.account_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(AccountLabelSlot => "worth.query.test.execution.governed_live.account_label_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.governed_live.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.governed_live.activity_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.execution.governed_live.activity_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedLiveAccountActivityResult {
    account: String,
    label: WorthQueryApplicationDisclosed<String>,
    activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(GovernedLiveAccountActivityResult => "worth.query.test.execution.governed_live.result.v1");

impl GovernedLiveAccountActivityResult {
    pub(in crate::domain_computation::primary_graph) fn account(&self) -> &str {
        &self.account
    }

    pub(in crate::domain_computation::primary_graph) const fn label(
        &self,
    ) -> &WorthQueryApplicationDisclosed<String> {
        &self.label
    }

    pub(in crate::domain_computation::primary_graph) fn activities(&self) -> &[(String, u64)] {
        &self.activities
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub GovernedLiveAccountActivityQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub GovernedLiveAccountActivityQueryResultBinding for GovernedLiveAccountActivityResult { identity: "worth.query.test.execution.governed_live.result.v1" });
worth_query_application_query!(
    pub GovernedLiveAccountActivityQuery for IdentityExecutionSchema,
    identity "GovernedLiveAccountActivityQuery",
    parameters GovernedLiveAccountActivityQueryParametersBinding,
    result GovernedLiveAccountActivityQueryResultBinding,
    scope Account => "Account",
    name "governed_live_account_activity"
);

pub(in crate::domain_computation::primary_graph) fn governed_live_account_parameters(
    account: impl Into<String>,
) -> ApplicationQueryParameterSet<GovernedLiveAccountActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(account_parameter(), account.into())
        .expect("fixture account identity must encode")
}

pub(super) fn governed_live_account_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    GovernedLiveAccountActivityQuery,
    AccountSummaryParameters,
    GovernedLiveAccountActivityResult,
    Account,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(activity_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        Account,
        GovernedLiveAccountActivityResult,
        GovernedLiveAccountActivityQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .field(account_label())
    .relation(activities(), activity)
    .build();
    let influence = ApplicationQueryInfluenceContract::permit_all();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "account-activity",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        AccountIdentity::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .use_field_by(
        ActivityIdentity::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .use_field_by(
        ActivitySequence::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .disclose_field_by(
        account_identity(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .disclose_field_by(
        account_label(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_relation_by(
        activities(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .disclose_field_by(
        activity_identity(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .disclose_field_by(
        activity_sequence(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence,
    );
    ApplicationQueryDefinitionBuilder::declare(GovernedLiveAccountActivityQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 4))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned().with_preview())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_preview()
                .with_live(),
        )
        .public()
        .parameter(account_parameter())
        .where_equal(AccountIdentity::reference(), account_parameter())
        .order_by(
            activity_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(activities())
        .live_by::<Activity, GovernedLiveAccountActivityCause, _, _, _, _, _, _, _, _>(
            account_identity(),
            activity_identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .unwrap()
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, GovernedLiveAccountActivityQuery>
    for GovernedLiveAccountActivityResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            GovernedLiveAccountActivityQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
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
            label: row.disclosed_field(account_label())?,
            activities,
        })
    }
}

pub struct GovernedLiveAccountActivityCause;
worth_query_declaration::worth_query_portable_type!(GovernedLiveAccountActivityCause => "worth.query.test.execution.governed_live.cause.v1");

impl
    ApplicationQueryLiveCauseBinding<
        IdentityExecutionSchema,
        GovernedLiveAccountActivityQuery,
        Account,
        Activity,
    > for GovernedLiveAccountActivityCause
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
        payload.account().to_owned()
    }

    fn target_identity(
        payload: &LiveActivityEvent,
    ) -> <Self::TargetIdentityBinding as ApplicationScalarValueBinding>::Value {
        payload.activity().to_owned()
    }
}

fn account_parameter() -> ApplicationQueryParameterRef<
    GovernedLiveAccountActivityQuery,
    AccountIdentityParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

fn account_identity() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
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

fn account_label() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
    AccountLabelSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountLabel,
    String,
    worth_query_declaration::facade::application_schema::ReadWrite,
    worth_query_declaration::facade::application_schema::EqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("label", AccountLabel::reference())
}

fn activity_identity() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
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
    GovernedLiveAccountActivityQuery,
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
    GovernedLiveAccountActivityQuery,
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
