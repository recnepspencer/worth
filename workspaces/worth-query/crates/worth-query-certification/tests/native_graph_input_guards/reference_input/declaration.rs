//! Assignment is a reference-only compiled body, not a custom resolver or mutation handler.
use super::super::{
    actions::{DenialBinding, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::{
    application_operation::NoApplicationMutationOutputs, application_schema::*,
    worth_query_mutation_binding, worth_query_operation, worth_query_operation_links,
    worth_query_operation_reads, worth_query_operation_unlinks, worth_query_relation,
    worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryOperationBodyDecision;
const fn nullable() -> ApplicationRelationIntegrity {
    let base = ApplicationRelationIntegrity::same_context_unbounded_retain_dangling();
    ApplicationRelationIntegrity::new(
        base.endpoints,
        ApplicationRelationCardinality::new(None, Some(1), None, None, None, None),
        base.deletion,
    )
}
worth_query_relation!(pub SelectedParent in Schema,Row=>Parent;integrity=nullable());
#[derive(Clone)]
pub struct Assign {
    pub row: String,
    pub selected: Option<String>,
    pub encoding: Encoding,
}
/// Exercises supplier serialization defects through original canonical capture.
#[derive(Clone, Copy)]
pub enum Encoding {
    Normal,
    Missing,
    WrongFamily,
}
impl serde::Serialize for Assign {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut record = serializer.serialize_struct(
            "Assign",
            if matches!(self.encoding, Encoding::Missing) {
                1
            } else {
                2
            },
        )?;
        record.serialize_field("row", &self.row)?;
        match self.encoding {
            Encoding::Normal => record.serialize_field("selected", &self.selected)?,
            Encoding::Missing => {}
            Encoding::WrongFamily => record.serialize_field("selected", &Some(42_u64))?,
        }
        record.end()
    }
}
worth_query_structured_value_binding!(pub Input for Assign {identity:"worth.query.reference.input.v1",capture_input:true});
worth_query_operation!(pub ReplaceReference for Schema,input Input);
worth_query_operation_reads!(ReplaceReference=>[Row,RowIdentity,Parent,ParentIdentity,SelectedParent]);
worth_query_operation_links!(ReplaceReference=>[SelectedParent]);
worth_query_operation_unlinks!(ReplaceReference=>[SelectedParent]);
fn row(input: &Assign) -> String {
    input.row.clone()
}
worth_query_mutation_binding!(pub Binding for Assign,schema Schema,
identity "worth.query.reference.binding.v1",input Input,operation ReplaceReference,result ResultBinding,
idempotency u64,identity "worth.query.reference.request.v1",decision WorthQueryOperationBodyDecision<Schema,Row>,denial DenialBinding,
handler identity "worth.query.reference.body.v1",program required,outputs NoApplicationMutationOutputs,
principal PrincipalBinding,mapping Mapping,principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
scope Row,RowFacts,RowIdentity,String,ReadOnly,NoApplicationUnit,field RowIdentity::reference(),value row,
);
pub fn body() -> ApplicationOperationBody<Schema, ReplaceReference, Assign> {
    ReplaceReference::reference()
        .body()
        .replace_relation_from_input(
            SelectedParent::reference(),
            ParentIdentity::reference(),
            "selected",
            "selected",
        )
        .unwrap()
        .finish()
        .unwrap()
}
pub fn declare(
    builder: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    super::signature_query::declare(super::signing::declare(builder))
        .relation(
            SelectedParent::reference(),
            Row::reference(),
            Parent::reference(),
        )
        .operation(
            ReplaceReference::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(body())
                .unwrap(),
        )
        .operation_read_field(ReplaceReference::reference(), RowIdentity::reference())
        .operation_projection_work_budget(ReplaceReference::reference(), 256)
        .application_mutation_binding::<Binding>()
        .application_query(super::query::definition())
        .application_query_binding::<super::query::ReadBinding>()
}
