//! Literal keyed graph readback includes fresh physical ids only for lifecycle assertions.
use super::*;
use worth_query_host::facade::application_invariants::EntityId as NativeEntityId;
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Snapshot {
    pub id: NativeEntityId,
    pub key: String,
    pub value: u64,
    pub head: Option<String>,
    pub members: Vec<(NativeEntityId, Row)>,
    pub notes: Vec<(NativeEntityId, String, u64, Option<u64>)>,
}
worth_query_structured_value_binding!(pub SnapshotBinding for Snapshot { identity: "source-subgraph.roster-snapshot.v1" });
worth_query_structured_value_binding!(pub Parameters for () { identity: "source-subgraph.roster-parameters.v1" });
worth_query_application_query!(pub Query for Schema, identity "source-subgraph.roster-query.v1", parameters Parameters, result SnapshotBinding, scope Source => "Source", name "roster");
macro_rules! slot { ($slot:ident, $function:ident, $field:ty, $name:literal) => {
    pub struct $slot; worth_query_portable_type!($slot => $name);
    fn $function() -> ApplicationQueryResultFieldRef<Query, $slot, Schema, $field> { ApplicationQueryResultFieldRef::new($name, <$field>::at($name)) }
}; }
slot!(RootKey, root_key, SourceKey, "SourceKey");
slot!(RootValue, root_value, SourceValue, "SourceValue");
slot!(ChildKey, child_key, MemberKey, "MemberKey");
slot!(ChildValue, child_value, Value, "MemberValue");
slot!(RemarkKey, remark_key, NoteKey, "NoteKey");
slot!(RemarkValue, remark_value, NoteValue, "NoteValue");
pub struct RemarkMark;
worth_query_portable_type!(RemarkMark => "source-subgraph.roster-note-mark.v1");
fn remark_mark() -> ApplicationQueryOptionalAspectFieldRef<Query, RemarkMark, Schema, Mark> {
    ApplicationQueryOptionalAspectFieldRef::new("mark", Mark::at("Mark"))
}
pub struct Notes;
worth_query_portable_type!(Notes => "source-subgraph.roster-notes.v1");
fn notes() -> ApplicationQueryResultRelationRef<
    Query,
    Notes,
    Schema,
    NotedBy,
    Note,
    Source,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("notes", NotedBy::reference())
}
pub struct NextKey;
worth_query_portable_type!(NextKey => "source-subgraph.roster-next-key.v1");
fn next_key() -> ApplicationQueryResultFieldRef<Query, NextKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("next_key", MemberKey::at("MemberKey"))
}
pub struct Members;
pub struct Successor;
worth_query_portable_type!(Members => "source-subgraph.roster-members.v1");
worth_query_portable_type!(Successor => "source-subgraph.roster-successor.v1");
fn members() -> ApplicationQueryResultRelationRef<
    Query,
    Members,
    Schema,
    OwnedBy,
    Member,
    Source,
    ReverseResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::reverse_many("members", OwnedBy::reference())
}
fn next() -> ApplicationQueryResultRelationRef<
    Query,
    Successor,
    Schema,
    Next,
    Member,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("next", Next::reference())
}
pub struct HeadKey;
worth_query_portable_type!(HeadKey => "source-subgraph.roster-head-key.v1");
fn head_key() -> ApplicationQueryResultFieldRef<Query, HeadKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("head_key", MemberKey::at("MemberKey"))
}
pub struct OperandKey;
worth_query_portable_type!(OperandKey => "source-subgraph.roster-operand-key.v1");
fn operand_key() -> ApplicationQueryResultFieldRef<Query, OperandKey, Schema, MemberKey> {
    ApplicationQueryResultFieldRef::new("operand_key", MemberKey::at("MemberKey"))
}
pub struct HeadMember;
worth_query_portable_type!(HeadMember => "source-subgraph.roster-head.v1");
fn head() -> ApplicationQueryResultRelationRef<
    Query,
    HeadMember,
    Schema,
    Head,
    Source,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("head", Head::reference())
}
pub struct OperandMember;
worth_query_portable_type!(OperandMember => "source-subgraph.roster-operand.v1");
fn operand() -> ApplicationQueryResultRelationRef<
    Query,
    OperandMember,
    Schema,
    Operand,
    Member,
    Member,
    ForwardResultTraversal,
    ManyResults,
> {
    ApplicationQueryResultRelationRef::forward_many("operand", Operand::reference())
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let next_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Snapshot,
        SnapshotBinding,
    >::new(Member::reference())
    .field(next_key());
    let member_shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Member,
        Snapshot,
        SnapshotBinding,
    >::new(Member::reference())
    .field(child_key())
    .field(child_value())
    .relation(operand(), ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Snapshot, SnapshotBinding>::new(Member::reference()).field(operand_key()))
    .relation(next(), next_shape);
    let shape = ApplicationQueryResultShapeBuilder::<
        Schema,
        Query,
        Source,
        Snapshot,
        SnapshotBinding,
    >::new(Source::reference())
    .field(root_key())
    .field(root_value())
    .relation(head(), ApplicationQueryResultShapeBuilder::<Schema, Query, Member, Snapshot, SnapshotBinding>::new(Member::reference()).field(head_key()))
    .relation(members(), member_shape)
    .relation(notes(), ApplicationQueryResultShapeBuilder::<Schema, Query, Note, Snapshot, SnapshotBinding>::new(Note::reference()).field(remark_key()).field(remark_value()).optional_aspect_field(remark_mark()))
    .build();
    schema
        .application_query(
            ApplicationQueryDefinitionBuilder::declare(Query::reference())
                .root(Source::reference())
                .scope(Source::reference())
                .result_shape(shape)
                .cardinality(ApplicationQueryCardinality::ExactlyOne)
                .disclosure(ApplicationQueryDisclosureContract::public())
                .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                .lanes(ApplicationQueryLaneEligibility::one_shot())
                .public(),
        )
        .application_query_binding::<Binding>()
}
impl WorthQueryApplicationProjection<Schema, Query> for Snapshot {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let mut members = row
            .many(members())?
            .iter()
            .map(|member| {
                let links = member.many(next())?;
                assert!(links.len() <= 1);
                Ok((
                    member.entity_id(),
                    Row {
                        key: member.field(child_key())?,
                        value: member.field(child_value())?,
                        operand: member
                            .many(operand())?
                            .iter()
                            .next()
                            .map(|target| target.field(operand_key()))
                            .transpose()?,
                        next: links
                            .iter()
                            .next()
                            .map(|next| next.field(next_key()))
                            .transpose()?,
                    },
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        members.sort_by(|a, b| a.1.key.cmp(&b.1.key));
        let mut notes = row
            .many(notes())?
            .iter()
            .map(|note| {
                Ok((
                    note.entity_id(),
                    note.field(remark_key())?,
                    note.field(remark_value())?,
                    note.optional_aspect_field(remark_mark())?,
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryApplicationProjectionDenial>>()?;
        notes.sort_by(|a, b| a.1.cmp(&b.1));
        Ok(Self {
            id: row.entity_id(),
            key: row.field(root_key())?,
            value: row.field(root_value())?,
            head: row
                .many(head())?
                .iter()
                .next()
                .map(|head| head.field(head_key()))
                .transpose()?,
            members,
            notes,
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Read(pub String);
worth_query_structured_value_binding!(pub ReadInput for Read { identity: "source-subgraph.roster-read-input.v1" });
fn scope(read: Read) -> String {
    read.0
}
worth_query_query_binding!(pub Binding for Read, schema Schema, identity "source-subgraph.roster-read.v1", input ReadInput,
    query Query, parameters Parameters => |_| ApplicationQueryParameterSet::new(), result SnapshotBinding,
    principal Actor, mapping Mapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope SourceKey, field SourceKey::at("SourceKey"), value scope, limits results 1);
