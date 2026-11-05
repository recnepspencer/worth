use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinition;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryOrderingDirection;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::facade::application_query::ExactlyOneResult;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::facade::application_query::ManyResults;
use worth_query_declaration::facade::application_query::OptionalOneResult;
use worth_query_declaration::facade::application_query::ReverseResultTraversal;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::parameter_reference::status_parameter;
use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountPrimaryActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountSecondaryActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::Activity;
use crate::domain_computation::primary_graph::tests::fixture::ActivityAccount;
use crate::domain_computation::primary_graph::tests::fixture::ActivityFacts;
use crate::domain_computation::primary_graph::tests::fixture::ActivitySequence;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;

pub struct PrimaryActivitySlot;
pub struct SecondaryActivitySlot;
pub struct AllActivitySlot;
pub struct ReverseActivitySlot;
pub struct PrimarySequenceSlot;
pub struct SecondarySequenceSlot;
pub struct AllSequenceSlot;
pub struct ReverseSequenceSlot;
worth_query_declaration::worth_query_portable_type!(PrimaryActivitySlot => "worth.query.test.execution.nested.primary_activity_slot.v1");
worth_query_declaration::worth_query_portable_type!(SecondaryActivitySlot => "worth.query.test.execution.nested.secondary_activity_slot.v1");
worth_query_declaration::worth_query_portable_type!(AllActivitySlot => "worth.query.test.execution.nested.all_activity_slot.v1");
worth_query_declaration::worth_query_portable_type!(ReverseActivitySlot => "worth.query.test.execution.nested.reverse_activity_slot.v1");
worth_query_declaration::worth_query_portable_type!(PrimarySequenceSlot => "worth.query.test.execution.nested.primary_sequence_slot.v1");
worth_query_declaration::worth_query_portable_type!(SecondarySequenceSlot => "worth.query.test.execution.nested.secondary_sequence_slot.v1");
worth_query_declaration::worth_query_portable_type!(AllSequenceSlot => "worth.query.test.execution.nested.all_sequence_slot.v1");
worth_query_declaration::worth_query_portable_type!(ReverseSequenceSlot => "worth.query.test.execution.nested.reverse_sequence_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NestedAccountResult {
    pub primary_activity: worth_relational::facade::identity::EntityId,
    pub secondary_activity: Option<worth_relational::facade::identity::EntityId>,
    pub all_activities: Vec<worth_relational::facade::identity::EntityId>,
    pub reverse_activities: Vec<worth_relational::facade::identity::EntityId>,
    pub primary_sequence: u64,
    pub secondary_sequence: Option<u64>,
    pub all_sequences: Vec<u64>,
    pub reverse_sequences: Vec<u64>,
}
worth_query_declaration::worth_query_portable_type!(NestedAccountResult => "worth.query.test.execution.nested.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub NestedUnitResultBinding for () { identity: "worth.rust.unit" });

impl NestedAccountResult {
    pub fn activity_identities(
        &self,
    ) -> (
        worth_relational::facade::identity::EntityId,
        Option<worth_relational::facade::identity::EntityId>,
        &[worth_relational::facade::identity::EntityId],
        &[worth_relational::facade::identity::EntityId],
    ) {
        (
            self.primary_activity,
            self.secondary_activity,
            &self.all_activities,
            &self.reverse_activities,
        )
    }

    pub const fn primary_sequence(&self) -> u64 {
        self.primary_sequence
    }

    pub const fn secondary_sequence(&self) -> Option<u64> {
        self.secondary_sequence
    }

    pub fn all_sequences(&self) -> &[u64] {
        &self.all_sequences
    }

    pub fn reverse_sequences(&self) -> &[u64] {
        &self.reverse_sequences
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub NestedAccountQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub NestedAccountQueryResultBinding for NestedAccountResult { identity: "worth.query.test.execution.nested.result.v1" });
worth_query_application_query!(
    pub NestedAccountQuery for IdentityExecutionSchema,
    identity "NestedAccountQuery",
    parameters NestedAccountQueryParametersBinding,
    result NestedAccountQueryResultBinding,
    scope Account => "Account",
    name "nested_account"
);

impl
    worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        NestedAccountQuery,
    > for NestedAccountResult
{
    fn project(
        row: &worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            NestedAccountQuery,
        >,
    ) -> Result<
        Self,
        worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial,
    > {
        let primary = row.one(primary_activity())?;
        let primary_activity = primary.entity_id();
        let primary_sequence = primary.field(primary_sequence())?;
        let secondary = row.optional(secondary_activity())?;
        let secondary_activity = secondary.as_ref().map(|activity| activity.entity_id());
        let secondary_sequence = secondary
            .map(|activity| activity.field(secondary_sequence()))
            .transpose()?;
        let all = row.many(all_activity())?;
        let all_activities = all.iter().map(|activity| activity.entity_id()).collect();
        let all_sequences = all
            .iter()
            .map(|activity| activity.field(all_sequence()))
            .collect::<Result<Vec<_>, _>>()?;
        let reverse = row.many(reverse_activity())?;
        let reverse_activities = reverse
            .iter()
            .map(|activity| activity.entity_id())
            .collect();
        let reverse_sequences = reverse
            .iter()
            .map(|activity| activity.field(reverse_sequence()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            primary_activity,
            secondary_activity,
            all_activities,
            reverse_activities,
            primary_sequence,
            secondary_sequence,
            all_sequences,
            reverse_sequences,
        })
    }
}

pub fn nested_account_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    NestedAccountQuery,
    AccountSummaryParameters,
    NestedAccountResult,
    Account,
> {
    let primary = nested_shape(primary_sequence());
    let secondary = nested_shape(secondary_sequence());
    let all = nested_shape(all_sequence());
    let reverse = nested_shape(reverse_sequence());
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        NestedAccountQuery,
        Account,
        NestedAccountResult,
        NestedAccountQueryResultBinding,
    >::new(Account::reference())
    .relation(primary_activity(), primary)
    .relation(secondary_activity(), secondary)
    .relation(all_activity(), all)
    .relation(reverse_activity(), reverse)
    .build();
    ApplicationQueryDefinitionBuilder::declare(NestedAccountQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .parameter(status_parameter())
        .where_equal(AccountStatus::reference(), status_parameter())
        .order_by(all_sequence(), ApplicationQueryOrderingDirection::Ascending)
        .build()
        .unwrap()
}

pub fn nested_shape<
    Slot: worth_query_declaration::facade::portable_identity::WorthQueryPortableType,
>(
    field: ApplicationQueryResultFieldRef<
        NestedAccountQuery,
        Slot,
        IdentityExecutionSchema,
        Activity,
        ActivityFacts,
        ActivitySequence,
        u64,
        worth_query_declaration::facade::application_schema::ReadOnly,
        worth_query_declaration::facade::application_schema::NoEqualityPredicate,
        worth_query_declaration::facade::application_schema::NoApplicationUnit,
    >,
) -> ApplicationQueryResultShapeBuilder<
    IdentityExecutionSchema,
    NestedAccountQuery,
    Activity,
    (),
    NestedUnitResultBinding,
> {
    ApplicationQueryResultShapeBuilder::new(Activity::reference()).field(field)
}

pub fn primary_sequence() -> ApplicationQueryResultFieldRef<
    NestedAccountQuery,
    PrimarySequenceSlot,
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

pub fn secondary_sequence() -> ApplicationQueryResultFieldRef<
    NestedAccountQuery,
    SecondarySequenceSlot,
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

pub fn all_sequence() -> ApplicationQueryResultFieldRef<
    NestedAccountQuery,
    AllSequenceSlot,
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

pub fn reverse_sequence() -> ApplicationQueryResultFieldRef<
    NestedAccountQuery,
    ReverseSequenceSlot,
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

pub fn primary_activity() -> ApplicationQueryResultRelationRef<
    NestedAccountQuery,
    PrimaryActivitySlot,
    IdentityExecutionSchema,
    AccountPrimaryActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    ExactlyOneResult,
> {
    ApplicationQueryResultRelationRef::forward_one(
        "primary_activity",
        AccountPrimaryActivity::reference(),
    )
}

pub fn secondary_activity() -> ApplicationQueryResultRelationRef<
    NestedAccountQuery,
    SecondaryActivitySlot,
    IdentityExecutionSchema,
    AccountSecondaryActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    OptionalOneResult,
> {
    ApplicationQueryResultRelationRef::forward_optional(
        "secondary_activity",
        AccountSecondaryActivity::reference(),
    )
}

pub fn all_activity() -> ApplicationQueryResultRelationRef<
    NestedAccountQuery,
    AllActivitySlot,
    IdentityExecutionSchema,
    AccountAllActivity,
    Account,
    Activity,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("all_activity", AccountAllActivity::reference())
}

pub fn reverse_activity() -> ApplicationQueryResultRelationRef<
    NestedAccountQuery,
    ReverseActivitySlot,
    IdentityExecutionSchema,
    ActivityAccount,
    Activity,
    Account,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many(
        "reverse_activity",
        ActivityAccount::reference(),
    )
}
