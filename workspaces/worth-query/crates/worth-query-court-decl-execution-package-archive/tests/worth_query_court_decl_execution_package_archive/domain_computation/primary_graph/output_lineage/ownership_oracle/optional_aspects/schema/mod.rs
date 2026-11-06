use super::*;
use worth_query_declaration::facade::application_schema::fragment::{self, *};
pub trait BirthSchema: TopologyModelBinding {
    type Nodes: Fragment;
    const INPUT_REUSE: bool = false;
    const RENEW_OUTPUT: bool = false;
    const SOURCE_WRITE: super::own_effect::SourceWrite = super::own_effect::SourceWrite::None;
}
pub struct NodeRecords<const REQUIRED: bool = false>;
impl<const R: bool> Fragment for NodeRecords<R>
where
    OutputNode<R>: Entity,
{
    const NAME: &'static str = "birth_nodes";
    type Bindings = NodeBindings;
    type Members = (OutputNode<R>, PresentNode, FieldlessNode, StrictNode);
}
pub struct NodeBindings;
impl BindingSet for NodeBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<String> for NodeBindings {
    type Binding = StringApplicationValueBinding;
}
#[derive(Clone, Copy, Debug)]
pub struct OutputNode<const REQUIRED: bool = false>;
impl Entity for OutputNode<false> {
    const NAME: &'static str = "OutputNode";
    type Aspects = (Option<NodeFacts>,);
    type Relations = (Incidence,);
}
impl Entity for OutputNode<true> {
    const NAME: &'static str = "OutputNode";
    type Aspects = (NodeFacts,);
    type Relations = (Incidence,);
}
impl OutputNode {
    pub fn reference<Schema: BirthSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(NodeRecords::<false>, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct PresentNode;
impl Entity for PresentNode {
    const NAME: &'static str = "PresentNode";
    type Aspects = (PresentFacts,);
    type Relations = ();
}
impl PresentNode {
    pub fn reference<Schema: BirthSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(NodeRecords::<false>, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct FieldlessNode;
impl Entity for FieldlessNode {
    const NAME: &'static str = "FieldlessNode";
    type Aspects = ();
    type Relations = ();
}
impl FieldlessNode {
    pub fn reference<Schema: BirthSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(NodeRecords::<false>, Self)
    }
}
#[derive(Clone, Copy, Debug)]
pub struct StrictNode;
impl Entity for StrictNode {
    const NAME: &'static str = "StrictNode";
    type Aspects = (Option<StrictFacts>,);
    type Relations = ();
}
impl StrictNode {
    pub fn reference<Schema: BirthSchema>() -> ApplicationEntityRef<Schema, Self> {
        entity(NodeRecords::<false>, Self)
    }
}
pub struct NodeFacts;
impl Aspect for NodeFacts {
    const NAME: &'static str = "NodeFacts";
    type Row = NodeValue;
}
pub struct PresentFacts;
impl Aspect for PresentFacts {
    const NAME: &'static str = "PresentFacts";
    type Row = PresentValue;
}
pub struct StrictFacts;
impl Aspect for StrictFacts {
    const NAME: &'static str = "StrictFacts";
    type Row = StrictValue;
}
pub struct NodeValue {
    note: Option<String>,
}
pub struct PresentValue {
    value: String,
}
pub struct StrictValue {
    mandatory: String,
    extra: Option<String>,
}
impl fragment::Record<NodeValue> for NodeBindings {
    fn record() -> impl Shape<NodeValue, Self> {
        field("Note", |row: &NodeValue| row.note.clone()).build(
            |note, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NodeValue { note })
            },
        )
    }
}
impl fragment::Record<PresentValue> for NodeBindings {
    fn record() -> impl Shape<PresentValue, Self> {
        field("Value", |row: &PresentValue| row.value.clone()).build(
            |value, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PresentValue { value })
            },
        )
    }
}
impl fragment::Record<StrictValue> for NodeBindings {
    fn record() -> impl Shape<StrictValue, Self> {
        field("Mandatory", |row: &StrictValue| row.mandatory.clone())
            .and("Extra", |row: &StrictValue| row.extra.clone())
            .build(|mandatory, extra, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(StrictValue { mandatory, extra })
            })
    }
}
pub struct SourceNotes;
impl Aspect for SourceNotes {
    const NAME: &'static str = "SourceNotes";
    type Row = SourceValue;
}
pub struct SourceValue {
    content: super::source_text::SourceText,
    selection: Option<super::source_text::SourceText>,
}
impl Scalar<super::source_text::SourceText> for TopologyRecordsBindings {
    type Binding = super::source_text::SourceTextBinding;
}
impl fragment::Record<SourceValue> for TopologyRecordsBindings {
    fn record() -> impl Shape<SourceValue, Self> {
        field("SourceContent", |row: &SourceValue| row.content.clone())
            .and("SourceSelection", |row: &SourceValue| row.selection.clone())
            .build(|content, selection, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(SourceValue { content, selection })
            })
    }
}
pub type Note = FieldSelection<NodeRecords, OutputNode, NodeFacts, String>;
pub type Value = FieldSelection<NodeRecords, PresentNode, PresentFacts, String>;
pub type Extra = FieldSelection<NodeRecords, StrictNode, StrictFacts, String>;
pub type SourceContent =
    FieldSelection<TopologyRecords, Body, SourceNotes, super::source_text::SourceText>;
pub type SourceSelection =
    FieldSelection<TopologyRecords, Body, SourceNotes, super::source_text::SourceText>;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Incidence;
impl Relation for Incidence {
    const NAME: &'static str = "Incidence";
    type To = FieldlessNode;
    const SELF_EDGES: bool = false;
}
impl Incidence {
    pub fn reference<Schema: BirthSchema>(
    ) -> ApplicationRelationRef<Schema, Self, OutputNode, FieldlessNode> {
        relation(NodeRecords::<false>, OutputNode, Self)
    }
}
pub fn declare<Schema: BirthSchema>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    operation::declare(schema)
}

impl SourceNotes {
    pub fn reference() -> AspectSelection<TopologyRecords, Body, Self> {
        aspect(TopologyRecords, Body, Self)
    }
}
