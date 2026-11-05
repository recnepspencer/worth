//! Native optional cells and ordered child rows use the ordinary typed projector.
use crate::domain_computation::primary_graph::tests::fixture::application_queries::label_result_field;
use crate::domain_computation::primary_graph::tests::fixture::diagnostic_comparison::*;
use crate::domain_computation::primary_graph::tests::fixture::AccountAllActivity;
use crate::domain_computation::primary_graph::tests::fixture::AccountNote;
use crate::domain_computation::primary_graph::tests::fixture::AccountPolicy;
use crate::domain_computation::primary_graph::tests::fixture::ActivityFacts;
use crate::domain_computation::primary_graph::tests::fixture::ActivityIdentity;
use worth_query_declaration::facade::application_query::ApplicationQueryOptionalResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultFieldRef;
use worth_query_declaration::facade::application_query::ApplicationQueryResultRelationRef;
use worth_query_declaration::facade::application_query::ForwardResultTraversal;
use worth_query_declaration::facade::application_query::ManyResults;
use worth_query_declaration::facade::application_schema::EqualityPredicate;
use worth_query_declaration::facade::application_schema::NoApplicationUnit;
use worth_query_declaration::facade::application_schema::NoEqualityPredicate;
use worth_query_declaration::facade::application_schema::ReadOnly;
use worth_query_declaration::facade::application_schema::ReadWrite;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjection;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionDenial;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationProjectionRow;

pub struct NoteSlot;
pub struct ActivitiesSlot;
pub struct IdentitySlot;
pub struct SequenceSlot;
worth_query_declaration::worth_query_portable_type!(NoteSlot => "worth.query.test.diagnostic.note.v1");
worth_query_declaration::worth_query_portable_type!(ActivitiesSlot => "worth.query.test.diagnostic.activities.v1");
worth_query_declaration::worth_query_portable_type!(IdentitySlot => "worth.query.test.diagnostic.identity.v1");
worth_query_declaration::worth_query_portable_type!(SequenceSlot => "worth.query.test.diagnostic.sequence.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticResult {
    pub label: String,
    pub note: Option<String>,
    pub activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(DiagnosticResult => "worth.query.test.diagnostic.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub DiagnosticResultBinding for DiagnosticResult { identity: "worth.query.test.diagnostic.result.v1" });
worth_query_declaration::worth_query_structured_value_binding!(pub DiagnosticUnitBinding for () { identity: "worth.rust.unit" });

pub fn label<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    crate::domain_computation::primary_graph::tests::fixture::application_queries::LabelResultSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    crate::domain_computation::primary_graph::tests::fixture::AccountLabel,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    label_result_field()
}

pub fn note<Q>() -> ApplicationQueryOptionalResultFieldRef<
    Q,
    NoteSlot,
    IdentityExecutionSchema,
    Account,
    AccountPolicy,
    AccountNote,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryOptionalResultFieldRef::new("note", AccountNote::reference())
}

pub fn activities<Q>() -> ApplicationQueryResultRelationRef<
    Q,
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

pub fn activity_identity<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    IdentitySlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivityIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", ActivityIdentity::reference())
}

pub fn sequence<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    SequenceSlot,
    IdentityExecutionSchema,
    Activity,
    ActivityFacts,
    ActivitySequence,
    u64,
    ReadOnly,
    NoEqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::reference())
}

impl<Q: ApplicationQueryMarkerIdentity<IdentityExecutionSchema>>
    WorthQueryApplicationProjection<IdentityExecutionSchema, Q> for DiagnosticResult
{
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, IdentityExecutionSchema, Q>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            label: row.field(label())?,
            note: row.optional_field(note())?,
            activities: row
                .many(activities())?
                .iter()
                .map(|child| Ok((child.field(activity_identity())?, child.field(sequence())?)))
                .collect::<Result<_, WorthQueryApplicationProjectionDenial>>()?,
        })
    }
}
