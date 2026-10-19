//! A current snapshot is row-owned, not a retained history record.
use super::schema::{Row, Schema};
use worth_query_decl::facade::{
    application_schema::*, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_relation,
};
worth_query_entity!(pub Snapshot for Schema);
worth_query_aspect!(pub SnapshotFacts for Schema, Snapshot; identity = AspectIdentity(0x9175_F001), revision = AspectContractRevision(1),);
worth_query_field!(pub SnapshotIdentity for Schema, Snapshot, SnapshotFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub SnapshotValue for Schema, Snapshot, SnapshotFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub SnapshotNote for Schema, Snapshot, SnapshotFacts: optional String => super::reuse_note_codec::NoteCodec, read_write, equality);
worth_query_field!(pub SnapshotLabel for Schema, Snapshot, SnapshotFacts: optional String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub SnapshotKind for Schema, Snapshot, SnapshotFacts: optional String => StringApplicationValueBinding, read_only, equality);
const fn ownership() -> ApplicationRelationIntegrity {
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    ApplicationRelationIntegrity::new(
        base.endpoints,
        ApplicationRelationCardinality::new(None, Some(1), None, Some(1), None, None),
        base.deletion,
    )
}
worth_query_relation!(pub CurrentSnapshot in Schema, Row => Snapshot; integrity = ownership());
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .entity(Snapshot::reference())
        .aspect(Snapshot::reference(), SnapshotFacts::reference())
        // A provider source row is not a global snapshot allocation identity.
        .field(Snapshot::reference(), SnapshotIdentity::reference())
        .field(Snapshot::reference(), SnapshotValue::reference())
        .field(Snapshot::reference(), SnapshotNote::reference())
        .field(Snapshot::reference(), SnapshotLabel::reference())
        .field(Snapshot::reference(), SnapshotKind::reference())
        .relation(
            CurrentSnapshot::reference(),
            Row::reference(),
            Snapshot::reference(),
        )
}
