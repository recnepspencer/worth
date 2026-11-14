//! General case transitions derive their inverse from captured fields and absence.
use super::*;
use worth_query_decl::facade::application_aftermath::*;
mod correction;
mod courts;
mod handler;
pub use correction::Correction;
pub use handler::Handler;

#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub enum Case {
    First { value: u64 },
    Second { value: u64, required: u64 },
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Input {
    pub key: String,
    pub case: Case,
}
worth_query_structured_value_binding!(pub InputValue for Input {identity: "native-record-patterns.sparse.input.v1"});
worth_query_operation!(pub Edit for Schema, input InputValue);
worth_query_operation_removes_aspects!(Edit => [First, Second, Third]);
pub struct Outputs;
pub struct Kept;
impl WorthQueryApplicationOutputRole for Kept {
    type Schema = Schema;
    type Contract = Outputs;
    type Entity = Record;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "kept-record";
}
impl ApplicationMutationOutputContract<Schema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Kept::DESCRIPTOR];
}
fn key(input: &Input) -> String {
    input.key.clone()
}
pub struct Decision {
    selected: ObservedTarget<Record>,
    output: WorthQueryInvariantMutationTarget<Schema, Record>,
}
worth_query_mutation_binding!(pub Binding for Input, schema Schema,
    identity "native-record-patterns.sparse.v1", input InputValue,
    operation Edit, result authoring::ResultBinding,
    idempotency String, identity "native-record-patterns.sparse.key.v1",
    decision Decision, denial authoring::DenialBinding,
    handler identity "native-record-patterns.sparse.handler.v1",
    program required, outputs Outputs,
    principal Actor, mapping Mapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key, field Key::at("Key"), value key,
);
impl ApplicationDeclaredOperation<Schema> for Edit {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> ApplicationSchemaDeclarationBuilder<Schema> {
        schema
            .operation_projection_work_budget(Self::reference(), 32)
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), FirstValue::at("FirstValue"))
            .operation_read_field(Self::reference(), SecondValue::at("SecondValue"))
            .operation_read_field(Self::reference(), SecondRequired::at("SecondRequired"))
            .operation_read_field(Self::reference(), ThirdValue::at("ThirdValue"))
            .operation_write(Self::reference(), FirstValue::at("FirstValue"))
            .operation_write(Self::reference(), SecondValue::at("SecondValue"))
            .operation_write(Self::reference(), SecondRequired::at("SecondRequired"))
            .operation_write(Self::reference(), ThirdValue::at("ThirdValue"))
            .operation_remove_aspect(Self::reference(), First::reference())
            .operation_remove_aspect(Self::reference(), Second::reference())
            .operation_remove_aspect(Self::reference(), Third::reference())
    }
}
pub fn loci() -> Vec<DeclaredPreImageLocus<Schema>> {
    vec![
        DeclaredPreImageLocus::from_field(Key::at("Key")),
        DeclaredPreImageLocus::from_field(FirstValue::at("FirstValue")),
        DeclaredPreImageLocus::from_field(SecondValue::at("SecondValue")),
        DeclaredPreImageLocus::from_field(SecondRequired::at("SecondRequired")),
        DeclaredPreImageLocus::from_field(ThirdValue::at("ThirdValue")),
    ]
}
