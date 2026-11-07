use worth_query_declaration::facade::application_query::ApplicationQueryBasisSupport;
use worth_query_declaration::facade::application_query::ApplicationQueryCardinality;
use worth_query_declaration::facade::application_query::ApplicationQueryDefinitionBuilder;
use worth_query_declaration::facade::application_query::ApplicationQueryDisclosureContract;
use worth_query_declaration::facade::application_query::ApplicationQueryLaneEligibility;
use worth_query_declaration::facade::application_query::ApplicationQueryOptionalResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultShapeBuilder;
use worth_query_declaration::worth_query_application_query;

use crate::domain_computation::primary_graph::tests::fixture::application_queries::AccountSummaryParameters;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountAnnotation;
use crate::domain_computation::primary_graph::tests::fixture::AccountIdentity;
use crate::domain_computation::primary_graph::tests::fixture::AccountNote;
use crate::domain_computation::primary_graph::tests::fixture::AccountScore;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;

pub struct AccountSlot;
pub struct NoteSlot;
pub struct ScoreSlot;
pub struct AnnotationSlot;
worth_query_declaration::worth_query_portable_type!(AccountSlot => "worth.query.test.execution.optional_account.account_slot.v1");
worth_query_declaration::worth_query_portable_type!(NoteSlot => "worth.query.test.execution.optional_account.note_slot.v1");
worth_query_declaration::worth_query_portable_type!(ScoreSlot => "worth.query.test.execution.optional_account.score_slot.v1");
worth_query_declaration::worth_query_portable_type!(AnnotationSlot => "worth.query.test.execution.optional_account.annotation_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptionalAccountFieldResult {
    pub account: String,
    pub note: Option<String>,
    pub score: Option<u64>,
    pub annotation: Option<String>,
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

    pub fn annotation(&self) -> Option<&str> {
        self.annotation.as_deref()
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

pub fn optional_account_field_definition() -> ApplicationQueryDefinitionBuilder<
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
    .optional_field(annotation())
    .build();
    ApplicationQueryDefinitionBuilder::declare(OptionalAccountFieldQuery::reference())
        .root(Account::reference())
        .scope(Account::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
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
            annotation: row.optional_field(annotation())?.map(Into::into),
        })
    }
}

pub fn account() -> ApplicationQueryResultFieldRef<
    OptionalAccountFieldQuery,
    AccountSlot,
    IdentityExecutionSchema,
    AccountIdentity,
> {
    ApplicationQueryResultFieldRef::new("account", AccountIdentity::at("AccountIdentity"))
}

pub fn note() -> ApplicationQueryOptionalResultFieldRef<
    OptionalAccountFieldQuery,
    NoteSlot,
    IdentityExecutionSchema,
    AccountNote,
> {
    ApplicationQueryOptionalResultFieldRef::new("note", AccountNote::at("AccountNote"))
}

pub fn score() -> ApplicationQueryOptionalResultFieldRef<
    OptionalAccountFieldQuery,
    ScoreSlot,
    IdentityExecutionSchema,
    AccountScore,
> {
    ApplicationQueryOptionalResultFieldRef::new("score", AccountScore::at("AccountScore"))
}

pub fn annotation() -> ApplicationQueryOptionalResultFieldRef<
    OptionalAccountFieldQuery,
    AnnotationSlot,
    IdentityExecutionSchema,
    AccountAnnotation,
> {
    ApplicationQueryOptionalResultFieldRef::new(
        "annotation",
        AccountAnnotation::at("AccountAnnotation"),
    )
}
