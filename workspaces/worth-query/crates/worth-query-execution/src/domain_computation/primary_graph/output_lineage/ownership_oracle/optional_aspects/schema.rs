use super::*;
pub(super) trait BirthSchema: TopologySchemaBinding {
    const REQUIRED: bool;
    const SOURCE_WRITE: super::own_effect::SourceWrite = super::own_effect::SourceWrite::None;
}
worth_query_entity!(pub(super) OutputNode for Schema: BirthSchema);
worth_query_entity!(pub(super) PresentNode for Schema: BirthSchema);
worth_query_entity!(pub(super) FieldlessNode for Schema: BirthSchema);
worth_query_entity!(pub(super) StrictNode for Schema: BirthSchema);
worth_query_aspect!(pub(super) NodeFacts for Schema: BirthSchema, OutputNode; identity = AspectIdentity(0x9174_3101), revision = AspectContractRevision(1));
worth_query_aspect!(pub(super) PresentFacts for Schema: BirthSchema, PresentNode; identity = AspectIdentity(0x9174_3102), revision = AspectContractRevision(1));
worth_query_aspect!(pub(super) StrictFacts for Schema: BirthSchema, StrictNode; identity = AspectIdentity(0x9174_3103), revision = AspectContractRevision(1));
worth_query_field!(pub(super) Note for Schema: BirthSchema, OutputNode, NodeFacts: optional String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) Value for Schema: BirthSchema, PresentNode, PresentFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) Mandatory for Schema: BirthSchema, StrictNode, StrictFacts: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) Extra for Schema: BirthSchema, StrictNode, StrictFacts: optional String => StringApplicationValueBinding, read_write, no_equality);
worth_query_aspect!(pub(super) SourceNotes for Schema: BirthSchema, Body; identity = AspectIdentity(0x9174_3104), revision = AspectContractRevision(1));
worth_query_field!(pub(super) SourceContent for Schema: BirthSchema, Body, SourceNotes: String => StringApplicationValueBinding, read_write, no_equality);
worth_query_field!(pub(super) SourceSelection for Schema: BirthSchema, Body, SourceNotes: optional String => StringApplicationValueBinding, read_write, no_equality);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Incidence;
impl Incidence {
    pub(super) const fn reference<Schema: BirthSchema>(
    ) -> ApplicationRelationRef<Schema, Self, OutputNode, FieldlessNode> {
        ApplicationRelationRef::from_schema_identifiers(
            "Incidence",
            "OutputNode",
            "FieldlessNode",
            ApplicationRelationIntegrity::same_context_no_self_edges_unbounded_retain_dangling(),
        )
    }
}

pub(super) fn declare<Schema: BirthSchema>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let schema = schema
        .entity(OutputNode::reference())
        .entity(PresentNode::reference())
        .entity(FieldlessNode::reference())
        .entity(StrictNode::reference());
    let schema = if Schema::REQUIRED {
        schema.aspect(OutputNode::reference(), NodeFacts::reference())
    } else {
        schema.optional_aspect(OutputNode::reference(), NodeFacts::reference())
    };
    let schema = schema
        .optional_aspect(Body::reference(), SourceNotes::reference())
        .field(Body::reference(), SourceContent::reference())
        .field(Body::reference(), SourceSelection::reference())
        .field(OutputNode::reference(), Note::reference())
        .aspect(PresentNode::reference(), PresentFacts::reference())
        .field(PresentNode::reference(), Value::reference())
        .optional_aspect(StrictNode::reference(), StrictFacts::reference())
        .field(StrictNode::reference(), Mandatory::reference())
        .field(StrictNode::reference(), Extra::reference())
        .relation(
            Incidence::reference(),
            OutputNode::reference(),
            FieldlessNode::reference(),
        );
    operation::declare(schema)
}
