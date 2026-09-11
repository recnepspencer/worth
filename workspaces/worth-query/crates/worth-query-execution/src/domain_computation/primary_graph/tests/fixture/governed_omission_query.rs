use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryInfluenceContract,
    ApplicationQueryLaneEligibility, ApplicationQueryOptionalResultFieldRef,
    ApplicationQueryResultFieldRef, ApplicationQueryResultRelationRef,
    ApplicationQueryResultShapeBuilder, ForwardResultTraversal, ManyResults,
    TypedApplicationQueryResultShape,
};
use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::worth_query_application_query;

use super::application_queries::AccountSummaryParameters;
use super::{
    Account, AccountAllActivity, AccountLabel, AccountNote, AccountPolicy, AccountStatus, Activity,
    ActivityFacts, ActivityIdentity, ActivitySequence, CapabilityDisclosure,
    CapabilityDisclosureBinding, IdentityExecutionSchema, TouchAccountCapability,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationDisclosed, WorthQueryApplicationProjection,
    WorthQueryApplicationProjectionDenial, WorthQueryApplicationProjectionRow,
};

worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

pub struct StatusSlot;
pub struct LabelSlot;
pub struct NoteSlot;
pub struct ActivitiesSlot;
pub struct ActivityIdentitySlot;
pub struct ActivitySequenceSlot;
worth_query_declaration::worth_query_portable_type!(StatusSlot => "worth.query.test.execution.governed_omission.status_slot.v1");
worth_query_declaration::worth_query_portable_type!(LabelSlot => "worth.query.test.execution.governed_omission.label_slot.v1");
worth_query_declaration::worth_query_portable_type!(NoteSlot => "worth.query.test.execution.governed_omission.note_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.execution.governed_omission.activities_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivityIdentitySlot => "worth.query.test.execution.governed_omission.activity_identity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ActivitySequenceSlot => "worth.query.test.execution.governed_omission.activity_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GovernedAccountOmissionResult {
    status: WorthQueryApplicationDisclosed<String>,
    label: WorthQueryApplicationDisclosed<String>,
    note: WorthQueryApplicationDisclosed<Option<String>>,
    activities: WorthQueryApplicationDisclosed<usize>,
}
worth_query_declaration::worth_query_portable_type!(GovernedAccountOmissionResult => "worth.query.test.execution.governed_omission.result.v1");

impl GovernedAccountOmissionResult {
    pub const fn status(&self) -> &WorthQueryApplicationDisclosed<String> {
        &self.status
    }

    pub const fn label(&self) -> &WorthQueryApplicationDisclosed<String> {
        &self.label
    }

    pub const fn note(&self) -> &WorthQueryApplicationDisclosed<Option<String>> {
        &self.note
    }

    pub const fn activities(&self) -> &WorthQueryApplicationDisclosed<usize> {
        &self.activities
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub GovernedAccountOmissionQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub GovernedAccountOmissionQueryResultBinding for GovernedAccountOmissionResult { identity: "worth.query.test.execution.governed_omission.result.v1" });
worth_query_application_query!(
    pub GovernedAccountOmissionQuery for IdentityExecutionSchema,
    identity "GovernedAccountOmissionQuery",
    parameters GovernedAccountOmissionQueryParametersBinding,
    result GovernedAccountOmissionQueryResultBinding,
    scope Account => "Account",
    name "governed_account_omission"
);

pub(super) fn governed_account_omission_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    GovernedAccountOmissionQuery,
    AccountSummaryParameters,
    GovernedAccountOmissionResult,
    Account,
> {
    let shape = governed_account_shape();
    let disclosure = governed_account_disclosure();
    ApplicationQueryDefinitionBuilder::declare(GovernedAccountOmissionQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 1, 5))
        .disclosure(disclosure)
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned().with_preview())
        .lanes(
            ApplicationQueryLaneEligibility::one_shot()
                .with_historical()
                .with_preview(),
        )
        .public()
        .build()
        .unwrap()
}

fn governed_account_shape() -> TypedApplicationQueryResultShape<
    IdentityExecutionSchema,
    GovernedAccountOmissionQuery,
    Account,
    GovernedAccountOmissionResult,
    GovernedAccountOmissionQueryResultBinding,
> {
    let activity = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedAccountOmissionQuery,
        Activity,
        (),
        NestedUnitResultBinding,
    >::new(Activity::reference())
    .field(activity_identity())
    .field(activity_sequence());
    ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        GovernedAccountOmissionQuery,
        Account,
        GovernedAccountOmissionResult,
        GovernedAccountOmissionQueryResultBinding,
    >::new(Account::reference())
    .field(status())
    .field(label())
    .optional_field(note())
    .relation(activities(), activity)
    .build()
}

fn governed_account_disclosure() -> ApplicationQueryDisclosureContract {
    ApplicationQueryDisclosureContract::governed_by(
        "account-omission",
        TouchAccountCapability::reference(),
    )
    .disclose_field_by(
        status(),
        encoded_disclosure(CapabilityDisclosure::AccountActivity),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_field_by(
        label(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_optional_field_by(
        note(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_relation_by(
        activities(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_field_by(
        activity_identity(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
    .disclose_field_by(
        activity_sequence(),
        encoded_disclosure(CapabilityDisclosure::PrivateLabel),
        ApplicationQueryInfluenceContract::forbid_all(),
    )
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, GovernedAccountOmissionQuery>
    for GovernedAccountOmissionResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            GovernedAccountOmissionQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let activities = match row.disclosed_many(activities())? {
            WorthQueryApplicationDisclosed::Disclosed(rows) => {
                WorthQueryApplicationDisclosed::Disclosed(rows.len())
            }
            WorthQueryApplicationDisclosed::Omitted(omission) => {
                WorthQueryApplicationDisclosed::Omitted(omission)
            }
        };
        Ok(Self {
            status: row.disclosed_field(status())?,
            label: row.disclosed_field(label())?,
            note: row.disclosed_optional_field(note())?,
            activities,
        })
    }
}

fn status() -> ApplicationQueryResultFieldRef<
    GovernedAccountOmissionQuery,
    StatusSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountStatus,
    String,
    worth_query_declaration::facade::application_schema::ReadWrite,
    worth_query_declaration::facade::application_schema::EqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("status", AccountStatus::reference())
}

fn label() -> ApplicationQueryResultFieldRef<
    GovernedAccountOmissionQuery,
    LabelSlot,
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

fn note() -> ApplicationQueryOptionalResultFieldRef<
    GovernedAccountOmissionQuery,
    NoteSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountNote,
    String,
    worth_query_declaration::facade::application_schema::ReadWrite,
    worth_query_declaration::facade::application_schema::EqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("note", AccountNote::reference())
}

fn activity_identity() -> ApplicationQueryResultFieldRef<
    GovernedAccountOmissionQuery,
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
    GovernedAccountOmissionQuery,
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
    GovernedAccountOmissionQuery,
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
