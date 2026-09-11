use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOrderingDirection, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ApplicationQueryResultShapeBuilder, ExactlyOneResult,
    ForwardResultTraversal, ManyResults, OptionalOneResult, ReverseResultTraversal,
};
use worth_query_declaration::worth_query_application_query;

use super::application_queries::{status_parameter, AccountSummaryParameters};
use super::{
    Account, AccountAllActivity, AccountPrimaryActivity, AccountSecondaryActivity, AccountStatus,
    Activity, ActivityAccount, ActivityFacts, ActivitySequence, IdentityExecutionSchema,
};

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
    primary_sequence: u64,
    secondary_sequence: Option<u64>,
    all_sequences: Vec<u64>,
    reverse_sequences: Vec<u64>,
}
worth_query_declaration::worth_query_portable_type!(NestedAccountResult => "worth.query.test.execution.nested.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(NestedUnitResultBinding for () { identity: "worth.rust.unit" });

impl NestedAccountResult {
    pub(in crate::domain_computation::primary_graph::tests) const fn primary_sequence(
        &self,
    ) -> u64 {
        self.primary_sequence
    }

    pub(in crate::domain_computation::primary_graph::tests) const fn secondary_sequence(
        &self,
    ) -> Option<u64> {
        self.secondary_sequence
    }

    pub(in crate::domain_computation::primary_graph::tests) fn all_sequences(&self) -> &[u64] {
        &self.all_sequences
    }

    pub(in crate::domain_computation::primary_graph::tests) fn reverse_sequences(&self) -> &[u64] {
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
    crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
        IdentityExecutionSchema,
        NestedAccountQuery,
    > for NestedAccountResult
{
    fn project(
        row: &crate::domain_computation::primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            NestedAccountQuery,
        >,
    ) -> Result<Self, crate::domain_computation::primary_graph::WorthQueryApplicationProjectionDenial>
    {
        let primary_sequence = row.one(primary_activity())?.field(primary_sequence())?;
        let secondary_sequence = row
            .optional(secondary_activity())?
            .map(|activity| activity.field(secondary_sequence()))
            .transpose()?;
        let all_sequences = row
            .many(all_activity())?
            .iter()
            .map(|activity| activity.field(all_sequence()))
            .collect::<Result<Vec<_>, _>>()?;
        let reverse_sequences = row
            .many(reverse_activity())?
            .iter()
            .map(|activity| activity.field(reverse_sequence()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            primary_sequence,
            secondary_sequence,
            all_sequences,
            reverse_sequences,
        })
    }
}

pub(super) fn nested_account_definition() -> ApplicationQueryDefinition<
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(1, 4, 4))
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

fn nested_shape<
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

fn primary_sequence() -> ApplicationQueryResultFieldRef<
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

fn secondary_sequence() -> ApplicationQueryResultFieldRef<
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

fn all_sequence() -> ApplicationQueryResultFieldRef<
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

fn reverse_sequence() -> ApplicationQueryResultFieldRef<
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

fn primary_activity() -> ApplicationQueryResultRelationRef<
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

fn secondary_activity() -> ApplicationQueryResultRelationRef<
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

fn all_activity() -> ApplicationQueryResultRelationRef<
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

fn reverse_activity() -> ApplicationQueryResultRelationRef<
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
