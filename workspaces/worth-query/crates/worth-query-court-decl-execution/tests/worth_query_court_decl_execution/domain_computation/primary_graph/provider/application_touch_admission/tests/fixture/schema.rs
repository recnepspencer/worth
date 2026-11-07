//! Native touch fixtures use one record binding for each meaning-bearing row.
use super::*;
use worth_query_decl::facade::application_schema::fragment::{self, *};

pub struct TouchSchema;
impl Fragment for TouchSchema {
    const NAME: &'static str = "TouchSchema";
    type Bindings = Bindings;
    type Members = (
        Node,
        RequiredNode,
        (OnlyA, super::ReadOnly),
        FieldsOnly,
        Both,
        relation::LinkOnly,
        relation::Initialize,
    );
}
impl ApplicationSchema for TouchSchema {
    const OWNER: &'static str = "struct_touch_admission";
    const NAME: &'static str = <Self as Fragment>::NAME;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
}
pub struct Bindings;
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        use_admission: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        use_admission(work)
    }
}
impl Scalar<String> for Bindings {
    type Binding = StringApplicationValueBinding;
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
pub struct Node;
impl Entity for Node {
    const NAME: &'static str = "Node";
    type Aspects = (Identity, Option<Notes>);
    type Relations = (WithAspects<relation::Edge, (Option<relation::Payload>,)>,);
}
pub struct RequiredNode;
impl Entity for RequiredNode {
    const NAME: &'static str = "RequiredNode";
    type Aspects = (RequiredNotes,);
    type Relations = ();
}
pub struct Identity;
impl Aspect for Identity {
    const NAME: &'static str = "Identity";
    type Row = KeyValue;
}
pub struct Notes;
impl Aspect for Notes {
    const NAME: &'static str = "Notes";
    type Row = NotesValue;
}
pub struct RequiredNotes;
impl Aspect for RequiredNotes {
    const NAME: &'static str = "RequiredNotes";
    type Row = NotesValue;
}
pub struct KeyValue {
    pub key: String,
}
pub struct NotesValue {
    pub a: Option<String>,
    pub b: Option<String>,
}
impl Record<KeyValue> for Bindings {
    fn record() -> impl Shape<KeyValue, Self> {
        field("Key", |row: &KeyValue| row.key.clone())
            .equality_queryable(true)
            .read_only()
            .build(|key, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(KeyValue { key })
            })
    }
}
impl Record<NotesValue> for Bindings {
    fn record() -> impl Shape<NotesValue, Self> {
        field("A", |row: &NotesValue| row.a.clone())
            .equality_queryable(true)
            .and("B", |row: &NotesValue| row.b.clone())
            .equality_queryable(true)
            .build(|a, b, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(NotesValue { a, b })
            })
    }
}
pub fn node() -> ApplicationEntityRef<TouchSchema, Node> {
    entity(TouchSchema, Node)
}
pub fn required_node() -> ApplicationEntityRef<TouchSchema, RequiredNode> {
    entity(TouchSchema, RequiredNode)
}
pub fn key_field() -> FieldSelection<TouchSchema, Node, Identity, String> {
    select_field(TouchSchema, Node, Identity, "Key")
}
pub fn a_field() -> FieldSelection<TouchSchema, Node, Notes, String> {
    select_field(TouchSchema, Node, Notes, "A")
}
pub fn b_field() -> FieldSelection<TouchSchema, Node, Notes, String> {
    select_field(TouchSchema, Node, Notes, "B")
}
pub fn required_a_field() -> FieldSelection<TouchSchema, RequiredNode, RequiredNotes, String> {
    select_field(TouchSchema, RequiredNode, RequiredNotes, "A")
}
pub fn required_b_field() -> FieldSelection<TouchSchema, RequiredNode, RequiredNotes, String> {
    select_field(TouchSchema, RequiredNode, RequiredNotes, "B")
}
pub fn notes() -> AspectSelection<TouchSchema, Node, Notes> {
    aspect(TouchSchema, Node, Notes)
}
pub fn notes_definition(
    schema: &WorthQueryInstalledApplicationSchema<TouchSchema>,
    required: bool,
) -> &AspectDefinition {
    let fragment = schema.value_bindings().fragment(TouchSchema).unwrap();
    if required {
        fragment.aspect::<RequiredNode, RequiredNotes>().unwrap()
    } else {
        fragment.aspect::<Node, Notes>().unwrap()
    }
}
pub fn notes_value(fields: &[(&str, &str)]) -> NotesValue {
    let mut row = NotesValue { a: None, b: None };
    for (field, value) in fields {
        match *field {
            "A" => row.a = Some((*value).to_owned()),
            "B" => row.b = Some((*value).to_owned()),
            _ => panic!("fixture names a declared field"),
        }
    }
    row
}
impl Registration<TouchSchema> for OnlyA {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_entity(operation, node())
            .operation_write(operation, a_field()))
    }
}
impl Registration<TouchSchema> for FieldsOnly {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_entity(operation, node())
            .operation_write(operation, a_field())
            .operation_write(operation, b_field()))
    }
}
impl Registration<TouchSchema> for Both {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        let operation = Self::reference();
        Ok(builder
            .operation(
                operation
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(operation, 16)
            .operation_read_entity(operation, node())
            .operation_read_entity(operation, required_node())
            .operation_read_field(operation, a_field())
            .operation_read_field(operation, b_field())
            .operation_write(operation, a_field())
            .operation_write(operation, b_field())
            .operation_write(operation, required_a_field())
            .operation_write(operation, required_b_field())
            .operation_remove_aspect(operation, notes()))
    }
}

impl Registration<TouchSchema> for super::ReadOnly {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<TouchSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<TouchSchema>, ApplicationSchemaDeclarationDenial>
    {
        Ok(builder
            .operation(
                super::ReadOnly::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_read_entity(super::ReadOnly::reference(), node()))
    }
}
