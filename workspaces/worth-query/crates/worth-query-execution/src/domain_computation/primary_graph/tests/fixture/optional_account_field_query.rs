use worth_query_declaration::facade::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    ApplicationQueryResultShapeBuilder,
};
use worth_query_declaration::worth_query_application_query;

use super::{
    Account, AccountIdentity, AccountNote, AccountPolicy, AccountScore, AccountSummaryParameters,
    IdentityExecutionSchema,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};

pub struct AccountSlot;
pub struct NoteSlot;
pub struct ScoreSlot;
worth_query_declaration::worth_query_portable_type!(AccountSlot => "worth.query.test.execution.optional_account.account_slot.v1");
worth_query_declaration::worth_query_portable_type!(NoteSlot => "worth.query.test.execution.optional_account.note_slot.v1");
worth_query_declaration::worth_query_portable_type!(ScoreSlot => "worth.query.test.execution.optional_account.score_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionalAccountFieldResult {
    account: String,
    note: Option<String>,
    score: Option<u64>,
}
worth_query_declaration::worth_query_portable_type!(OptionalAccountFieldResult => "worth.query.test.execution.optional_account.result.v1");

impl OptionalAccountFieldResult {
    pub fn account(&self) -> &str {
        &self.account
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub const fn score(&self) -> Option<u64> {
        self.score
    }
}

worth_query_declaration::worth_query_structured_value_binding!(pub OptionalAccountFieldQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub OptionalAccountFieldQueryResultBinding for OptionalAccountFieldResult { identity: "worth.query.test.execution.optional_account.result.v1" });
worth_query_application_query!(
    pub OptionalAccountFieldQuery for IdentityExecutionSchema,
    identity "OptionalAccountFieldQuery",
    parameters OptionalAccountFieldQueryParametersBinding,
    result OptionalAccountFieldQueryResultBinding,
    scope Account => "Account",
    name "optional_account_field"
);

pub(super) fn optional_account_field_definition() -> ApplicationQueryDefinition<
    IdentityExecutionSchema,
    OptionalAccountFieldQuery,
    AccountSummaryParameters,
    OptionalAccountFieldResult,
    Account,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        IdentityExecutionSchema,
        OptionalAccountFieldQuery,
        Account,
        OptionalAccountFieldResult,
        OptionalAccountFieldQueryResultBinding,
    >::new(Account::reference())
    .field(account())
    .optional_field(note())
    .optional_field(score())
    .build();
    ApplicationQueryDefinitionBuilder::declare(OptionalAccountFieldQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 3))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the optional field fixture is statically canonical")
}

impl WorthQueryApplicationProjection<IdentityExecutionSchema, OptionalAccountFieldQuery>
    for OptionalAccountFieldResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<
            '_,
            IdentityExecutionSchema,
            OptionalAccountFieldQuery,
        >,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            account: row.field(account())?,
            note: row.optional_field(note())?,
            score: row.optional_field(score())?,
        })
    }
}

fn account() -> ApplicationQueryResultFieldRef<
    OptionalAccountFieldQuery,
    AccountSlot,
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

fn note() -> ApplicationQueryOptionalResultFieldRef<
    OptionalAccountFieldQuery,
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

fn score() -> ApplicationQueryOptionalResultFieldRef<
    OptionalAccountFieldQuery,
    ScoreSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountScore,
    u64,
    worth_query_declaration::facade::application_schema::ReadWrite,
    worth_query_declaration::facade::application_schema::NoEqualityPredicate,
    worth_query_declaration::facade::application_schema::NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("score", AccountScore::reference())
}
