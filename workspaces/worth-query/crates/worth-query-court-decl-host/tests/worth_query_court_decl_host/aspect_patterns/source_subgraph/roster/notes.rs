//! Notes are a second owned kind: each belongs to one roster and links to nothing else.
use super::*;
use worth_query_decl::facade::application_schema::fragment::*;
pub struct Note;
impl Entity for Note {
    const NAME: &'static str = "AuthoredNote";
    type Aspects = (NoteFacts,);
    type Relations = (NotedBy,);
}
impl Note {
    pub fn reference() -> ApplicationEntityRef<Schema, Self> {
        entity(SourceModel, Self)
    }
}
pub struct NoteFacts;
impl Aspect for NoteFacts {
    const NAME: &'static str = "NoteFacts";
    type Row = NoteFactsValue;
}
pub struct NoteFactsValue {
    pub key: String,
    pub value: u64,
}
impl Record<NoteFactsValue> for SourceModelBindings {
    fn record() -> impl Shape<NoteFactsValue, Self> {
        field("NoteKey", |row: &NoteFactsValue| row.key.clone())
            .equality_queryable(true)
            .and("NoteValue", |row: &NoteFactsValue| row.value)
            .build(|key, value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NoteFactsValue { key, value })
            })
    }
}
pub type NoteKey = FieldSelection<SourceModel, Note, NoteFacts, String>;
pub type NoteValue = FieldSelection<SourceModel, Note, NoteFacts, u64>;
/// A note carries a mark or none; a deleted note returns exactly as it was.
pub struct NoteMark;
impl Aspect for NoteMark {
    const NAME: &'static str = "NoteMark";
    type Row = NoteMarkValue;
}
pub struct NoteMarkValue {
    pub mark: u64,
}
impl Record<NoteMarkValue> for SourceModelBindings {
    fn record() -> impl Shape<NoteMarkValue, Self> {
        field("Mark", |row: &NoteMarkValue| row.mark).build(
            |mark, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NoteMarkValue { mark })
            },
        )
    }
}
pub type Mark = FieldSelection<SourceFeature, Note, NoteMark, u64>;
pub struct NotedBy;
impl Relation for NotedBy {
    const NAME: &'static str = "NotedBy";
    type To = Source;
    const CARDINALITY: ApplicationRelationCardinality =
        ApplicationRelationCardinality::new(Some(1), Some(1), None, None, None, None);
    const DELETION: ApplicationRelationDeletionPolicy =
        ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations;
}
impl NotedBy {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, Note, Source> {
        relation(SourceModel, Note, Self)
    }
}
pub struct NoteFamily;
impl ApplicationAuthoredCorrectionFamily<Schema> for NoteFamily {
    const IDENTITY: &'static str = "source-subgraph.note.v1";
    type Entity = Note;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Note> {
        ApplicationAuthoredLogicalKey::from_field(NoteKey::at("NoteKey"))
    }
}
