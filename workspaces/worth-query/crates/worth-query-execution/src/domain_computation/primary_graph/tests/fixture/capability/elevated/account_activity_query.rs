use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility, ApplicationQueryLiveCauseBinding,
    ApplicationQueryLiveResourceContract, ApplicationQueryOrderingDirection,
    ApplicationQueryParameterRef, ApplicationQueryParameterSet, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ForwardResultTraversal,
    ManyResults, TypedApplicationQueryResultShape,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationEncodedScalarValue, ApplicationScalarValueBinding, StringApplicationValueBinding,
};
use worth_query_declaration::worth_query_application_query;

use super::super::super::application_queries::AccountSummaryParameters;
use super::super::super::live_account_query::{
    LiveActivityEffect, LiveActivityEvent, LiveActivityEventBinding,
};
use super::super::super::{
    Account, AccountAllActivity, AccountIdentity, AccountPolicy, Activity, ActivityFacts,
    ActivityIdentity, ActivitySequence, CapabilityDisclosure, CapabilityDisclosureBinding,
    IdentityExecutionSchema,
};
use super::ElevatedTouchAccountCapability;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct AccountIdentitySlot;
pub struct AccountIdentityParameter;
pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;
pub struct ActivitySequenceSlot;

worth_query_declaration::worth_query_portable_type!(AccountIdentitySlot => "worth.query.test.execution.elevated.account_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.elevated.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.elevated.activity_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.execution.elevated.activity_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ElevatedAccountActivityResult {
    account: String,
    activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(ElevatedAccountActivityResult => "worth.query.test.execution.elevated.result.v1");

impl ElevatedAccountActivityResult {
    pub(in crate::domain_computation::primary_graph) fn account(&self) -> &str {
        &self.account
    }

    pub(in crate::domain_computation::primary_graph) fn activities(&self) -> &[(String, u64)] {
        &self.activities
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub ElevatedAccountActivityQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ElevatedAccountActivityQueryResultBinding for ElevatedAccountActivityResult { identity: "worth.query.test.execution.elevated.result.v1" });
worth_query_application_query!(
    pub ElevatedAccountActivityQuery for IdentityExecutionSchema,
    identity "ElevatedAccountActivityQuery",
    parameters ElevatedAccountActivityQueryParametersBinding,
    result ElevatedAccountActivityQueryResultBinding,
    scope Account => "Account",
    name "elevated_account_activity"
);

pub(in crate::domain_computation::primary_graph) fn elevated_account_activity_parameters(
    account: impl Into<String>,
) -> ApplicationQueryParameterSet<ElevatedAccountActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(account_parameter(), account.into())
        .expect("fixture account identity must encode")
}

pub(in crate::domain_computation::primary_graph) fn elevated_account_activity_definition(
) -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    ElevatedAccountActivityQuery,
    AccountSummaryParameters,
    ElevatedAccountActivityResult,
    Account,
> {
    let shape = result_shape();
    let disclosure = disclosure_contract();
    ApplicationQueryDefinitionBuilder::declare(ElevatedAccountActivityQuery::reference())
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
        .live_by::<Activity, ElevatedAccountActivityCause, _, _, _, _, _, _, _, _>(
            account_identity(),
            activity_identity(),
            ApplicationQueryLiveResourceContract::bounded(4, 2_048, 4_096),
        )
        .build()
        .unwrap()
}

fn result_shape() -> TypedApplicationQueryResultShape<
    IdentityExecutionSchema,
    ElevatedAccountActivityQuery,
    Account,
    ElevatedAccountActivityResult,
    ElevatedAccountActivityQueryResultBinding,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ElevatedAccountActivityQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(activity_sequence());
    ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        ElevatedAccountActivityQuery,
        Account,
        ElevatedAccountActivityResult,
        ElevatedAccountActivityQueryResultBinding,
    >::new(Account::reference())
    .field(account_identity())
    .relation(activities(), activity)
    .build()
}

fn disclosure_contract() -> ApplicationQueryDisclosureContract {
    let influence = ApplicationQueryInfluenceContract::permit_all();
    ApplicationQueryDisclosureContract::governed_by(
        "elevated-account-activity",
        ElevatedTouchAccountCapability::reference(),
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
    )
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, ElevatedAccountActivityQuery>
    for ElevatedAccountActivityResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            ElevatedAccountActivityQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
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
            account: row.field(account_identity())?,
            activities,
        })
    }
}

pub struct ElevatedAccountActivityCause;
worth_query_declaration::worth_query_portable_type!(ElevatedAccountActivityCause => "worth.query.test.execution.elevated.live_cause.v1");

impl
    ApplicationQueryLiveCauseBinding<
        IdentityExecutionSchema,
        ElevatedAccountActivityQuery,
        Account,
        Activity,
    > for ElevatedAccountActivityCause
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
    ElevatedAccountActivityQuery,
    AccountIdentityParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

fn account_identity() -> ApplicationQueryResultFieldRef<
    ElevatedAccountActivityQuery,
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
    ElevatedAccountActivityQuery,
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
    ElevatedAccountActivityQuery,
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
    ElevatedAccountActivityQuery,
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
