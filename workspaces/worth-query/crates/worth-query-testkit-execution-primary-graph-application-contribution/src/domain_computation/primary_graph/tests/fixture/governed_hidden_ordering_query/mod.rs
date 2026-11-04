use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryInfluenceContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryObservableInfluence;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::facade::application_query::ManyResults;
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivityFacts;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::CapabilityDisclosureBinding;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationDisclosed;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;
pub struct ActivitySequenceSlot;
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.governed_hidden.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.governed_hidden.activity_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.execution.governed_hidden.activity_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedHiddenOrderingActivity {
    pub identity: WorthQueryApplicationDisclosed<String>,
    pub sequence: WorthQueryApplicationDisclosed<u64>,
    pub required_sequence_denial: WorthQueryApplicationProjectionDenialKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedHiddenOrderingResult {
    pub activities: WorthQueryApplicationDisclosed<Vec<GovernedHiddenOrderingActivity>>,
}
worth_query_declaration::worth_query_portable_type!(GovernedHiddenOrderingResult => "worth.query.test.execution.governed_hidden.result.v1");

impl GovernedHiddenOrderingActivity {
    pub const fn identity(&self) -> &WorthQueryApplicationDisclosed<String> {
        &self.identity
    }

    pub const fn sequence(&self) -> &WorthQueryApplicationDisclosed<u64> {
        &self.sequence
    }

    pub const fn required_sequence_denial(&self) -> WorthQueryApplicationProjectionDenialKind {
        self.required_sequence_denial
    }
}

impl GovernedHiddenOrderingResult {
    pub const fn activities(
        &self,
    ) -> &WorthQueryApplicationDisclosed<Vec<GovernedHiddenOrderingActivity>> {
        &self.activities
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub GovernedHiddenOrderingQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub GovernedHiddenOrderingQueryResultBinding for GovernedHiddenOrderingResult { identity: "worth.query.test.execution.governed_hidden.result.v1" });
worth_query_application_query!(
    pub GovernedHiddenOrderingQuery for IdentityExecutionSchema,
    identity "GovernedHiddenOrderingQuery",
    parameters GovernedHiddenOrderingQueryParametersBinding,
    result GovernedHiddenOrderingQueryResultBinding,
    scope Account => "Account",
    name "governed_hidden_ordering"
);

pub fn governed_hidden_ordering_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    GovernedHiddenOrderingQuery,
    AccountSummaryParameters,
    GovernedHiddenOrderingResult,
    Account,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedHiddenOrderingQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(activity_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedHiddenOrderingQuery,
        Account,
        GovernedHiddenOrderingResult,
        GovernedHiddenOrderingQueryResultBinding,
    >::new(Account::reference())
    .relation(activities(), activity)
    .build();
    let disclosure = ApplicationQueryDisclosureContract::governed_by(
        "hidden-ordering",
        TouchAccountCapability::reference(),
    )
    .use_field_by(
        ActivitySequence::reference(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit([ApplicationQueryObservableInfluence::Ordering]),
    )
    .disclose_relation_by(
        activities(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        activity_identity(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::permit_all(),
    )
    .disclose_field_by(
        activity_sequence(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    );
    ApplicationQueryDefinitionBuilder::declare(GovernedHiddenOrderingQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .order_by(
            activity_sequence(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .unwrap()
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, GovernedHiddenOrderingQuery>
    for GovernedHiddenOrderingResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            GovernedHiddenOrderingQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let activities = match row.disclosed_many(activities())? {
            WorthQueryApplicationDisclosed::Omitted(omission) => {
                WorthQueryApplicationDisclosed::Omitted(omission)
            }
            WorthQueryApplicationDisclosed::Disclosed(rows) => {
                let activities = rows
                    .iter()
                    .map(|activity| {
                        let required_sequence_denial = activity
                            .field(activity_sequence())
                            .expect_err("internal ordering material must not remain projected")
                            .kind();
                        Ok(GovernedHiddenOrderingActivity {
                            identity: activity.disclosed_field(activity_identity())?,
                            sequence: activity.disclosed_field(activity_sequence())?,
                            required_sequence_denial,
                        })
                    })
                    .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
                WorthQueryApplicationDisclosed::Disclosed(activities)
            }
        };
        Ok(Self { activities })
    }
}

pub fn activity_identity() -> ApplicationQueryResultFieldRef<
    GovernedHiddenOrderingQuery,
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
    ApplicationQueryResultFieldRef::new("identity", ActivityIdentity::reference())
}

pub fn activity_sequence() -> ApplicationQueryResultFieldRef<
    GovernedHiddenOrderingQuery,
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

pub fn activities() -> ApplicationQueryResultRelationRef<
    GovernedHiddenOrderingQuery,
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
