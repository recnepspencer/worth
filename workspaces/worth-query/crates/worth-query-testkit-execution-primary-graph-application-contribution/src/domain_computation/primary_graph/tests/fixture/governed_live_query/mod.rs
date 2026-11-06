use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
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
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::ApplicationScalarValueBinding;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::live_account_query::LiveActivityEffect;
use crate::domain_computation::primary_graph::tests::fixture::live_account_query::LiveActivityEvent;
use crate::domain_computation::primary_graph::tests::fixture::live_account_query::LiveActivityEventBinding;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;

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
    pub account: String,
    pub label: WorthQueryApplicationDisclosed<String>,
    pub activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(GovernedLiveAccountActivityResult => "worth.query.test.execution.governed_live.result.v1");

impl GovernedLiveAccountActivityResult {
    pub fn account(&self) -> &str {
        &self.account
    }

    pub const fn label(&self) -> &WorthQueryApplicationDisclosed<String> {
        &self.label
    }

    pub fn activities(&self) -> &[(String, u64)] {
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

pub fn governed_live_account_parameters(
    account: impl Into<String>,
) -> ApplicationQueryParameterSet<GovernedLiveAccountActivityQuery> {
    ApplicationQueryParameterSet::new()
        .bind(account_parameter(), account.into())
        .expect("fixture account identity must encode")
}

pub fn governed_live_account_definition() -> ApplicationQueryDefinition<
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
        AccountIdentity::at("AccountIdentity"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .expect("record field contract")
    .use_field_by(
        ActivityIdentity::at("ActivityIdentity"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .expect("record field contract")
    .use_field_by(
        ActivitySequence::at("ActivitySequence"),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        influence.clone(),
    )
    .expect("record field contract")
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
        .where_equal(AccountIdentity::at("AccountIdentity"), account_parameter())
        .order_by(
            activity_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .continue_by(activities())
        .live_by::<Activity, GovernedLiveAccountActivityCause>(
            account_identity(),
            activity_identity(),
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

pub fn account_parameter() -> ApplicationQueryParameterRef<
    GovernedLiveAccountActivityQuery,
    AccountIdentityParameter,
    StringApplicationValueBinding,
> {
    ApplicationQueryParameterRef::from_query_identifier("account")
}

pub fn account_identity() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
    AccountIdentitySlot,
    IdentityExecutionSchema,
    AccountIdentity,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

pub fn account_label() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
    AccountLabelSlot,
    IdentityExecutionSchema,
    AccountLabel,
> {
    ApplicationQueryResultFieldRef::new("label", AccountLabel::at("AccountLabel"))
}

pub fn activity_identity() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
    ActivityIdentitySlot,
    IdentityExecutionSchema,
    ActivityIdentity,
> {
    ApplicationQueryResultFieldRef::new("activity", ActivityIdentity::at("ActivityIdentity"))
}

pub fn activity_sequence() -> ApplicationQueryResultFieldRef<
    GovernedLiveAccountActivityQuery,
    ActivitySequenceSlot,
    IdentityExecutionSchema,
    ActivitySequence,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
}

pub fn activities() -> ApplicationQueryResultRelationRef<
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

pub fn encoded_disclosure(
    value: CapabilityDisclosure,
) -> ApplicationEncodedScalarValue<CapabilityDisclosureBinding> {
    ApplicationEncodedScalarValue::try_new(value).expect("fixture disclosure must encode")
}
