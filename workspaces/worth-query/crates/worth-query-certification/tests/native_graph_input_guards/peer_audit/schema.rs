use super::super::child_actions::Detail;
use super::super::schema::{Parent, Schema};
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_relation,
};
worth_query_entity!(pub Journal for Schema);
worth_query_aspect!(pub JournalFacts for Schema, Journal; identity = AspectIdentity(0x9175_F050), revision=AspectContractRevision(1),);
worth_query_field!(pub JournalName for Schema, Journal, JournalFacts: String=>StringApplicationValueBinding,read_only,equality);
worth_query_field!(pub JournalRevision for Schema, Journal, JournalFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_field!(pub JournalActor for Schema, Journal, JournalFacts: u64=>U64ApplicationValueBinding,read_only,equality);
worth_query_relation!(pub ParentJournal in Schema,Parent=>Journal;integrity=ApplicationRelationIntegrity::same_context_unbounded_retain_dangling());
const fn one_detail() -> ApplicationRelationIntegrity {
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    ApplicationRelationIntegrity::new(
        base.endpoints,
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, None),
        base.deletion,
    )
}
worth_query_relation!(pub JournalDetails in Schema,Journal=>Detail;integrity=one_detail());
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    builder
        .entity(Journal::reference())
        .aspect(Journal::reference(), JournalFacts::reference())
        .field(Journal::reference(), JournalName::reference())
        .field(Journal::reference(), JournalRevision::reference())
        .field(Journal::reference(), JournalActor::reference())
        .relation(
            JournalDetails::reference(),
            Journal::reference(),
            Detail::reference(),
        )
        .relation(
            ParentJournal::reference(),
            Parent::reference(),
            Journal::reference(),
        )
}
