//! Native optional cells and ordered child rows use the ordinary typed projector.
use super::super::application_queries::label_result_field;
use super::super::{AccountAllActivity, AccountNote, ActivityIdentity};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationProjectionDenial,
    WorthQueryApplicationProjectionRow,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryOptionalResultFieldRef, ApplicationQueryResultFieldRef,
    ApplicationQueryResultRelationRef, ForwardResultTraversal, ManyResults,
};

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
    pub(in crate::domain_computation::primary_graph) label: String,
    pub(in crate::domain_computation::primary_graph) note: Option<String>,
    pub(in crate::domain_computation::primary_graph) activities: Vec<(String, u64)>,
}
worth_query_declaration::worth_query_portable_type!(DiagnosticResult => "worth.query.test.diagnostic.result.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub DiagnosticResultBinding for DiagnosticResult { identity: "worth.query.test.diagnostic.result.v1" });
worth_query_declaration::worth_query_structured_value_binding!(pub(super) DiagnosticUnitBinding for () { identity: "worth.rust.unit" });

pub(super) fn label<Q>() -> ApplicationQueryResultFieldRef<
    Q,
    super::super::application_queries::LabelResultSlot,
    IdentityExecutionSchema,
    super::super::AccountLabel,
> {
    label_result_field()
}

pub(super) fn note<Q>(
) -> ApplicationQueryOptionalResultFieldRef<Q, NoteSlot, IdentityExecutionSchema, AccountNote> {
    ApplicationQueryOptionalResultFieldRef::new("note", AccountNote::at("AccountNote"))
}

pub(super) fn activities<Q>() -> ApplicationQueryResultRelationRef<
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

pub(super) fn activity_identity<Q>(
) -> ApplicationQueryResultFieldRef<Q, IdentitySlot, IdentityExecutionSchema, ActivityIdentity> {
    ApplicationQueryResultFieldRef::new("identity", ActivityIdentity::at("ActivityIdentity"))
}

pub(super) fn sequence<Q>(
) -> ApplicationQueryResultFieldRef<Q, SequenceSlot, IdentityExecutionSchema, ActivitySequence> {
    ApplicationQueryResultFieldRef::new("sequence", ActivitySequence::at("ActivitySequence"))
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
