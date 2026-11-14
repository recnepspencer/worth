//! Complete sole-root retirement retains field absence without borrowing collateral truth.
use super::*;
use sparse::annotation::{Annotation, AnnotationNote, AnnotationValue};
use worth_query_decl::facade::application_aftermath::*;
pub mod correction;
mod courts;
pub mod restore;
pub mod retire;
pub struct Family;
impl ApplicationAuthoredCorrectionFamily<Schema> for Family {
    const IDENTITY: &'static str = "native-record-patterns.complete-family.v1";
    type Entity = Record;
    fn logical_key() -> ApplicationAuthoredLogicalKey<Schema, Record> {
        ApplicationAuthoredLogicalKey::from_field(Key::at("Key"))
    }
}
#[derive(Clone, Debug, WorthValue)]
#[worth_value(crate = "worth_query_decl::facade::application_schema")]
pub struct Image {
    key: String,
    row: query::Row,
    annotation: Option<u64>,
    note: Option<u64>,
}
pub fn loci() -> Vec<DeclaredPreImageLocus<Schema>> {
    vec![
        DeclaredPreImageLocus::from_field(Key::at("Key")),
        DeclaredPreImageLocus::from_field(FirstValue::at("FirstValue")),
        DeclaredPreImageLocus::from_field(SecondValue::at("SecondValue")),
        DeclaredPreImageLocus::from_field(SecondRequired::at("SecondRequired")),
        DeclaredPreImageLocus::from_field(ThirdValue::at("ThirdValue")),
        DeclaredPreImageLocus::from_field(AnnotationValue::at("Value")),
        DeclaredPreImageLocus::from_field(AnnotationNote::at("Note")),
    ]
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    schema
        .application_mutation_binding::<retire::Binding>()
        .application_mutation_binding::<restore::Binding>()
}
