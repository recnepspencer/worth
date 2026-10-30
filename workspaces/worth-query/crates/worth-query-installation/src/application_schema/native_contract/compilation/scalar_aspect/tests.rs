//! Explicit storage changes meaning; invalid field closure cannot become Scalar.
use super::*;
use worth_query_declaration::facade::application_schema::*;
mod installed_read_scope;
worth_query_declaration::worth_query_application_schema! {
    schema ScalarSchema {
        owner: ScalarSchema, version: (1, 0),
        members: |schema| { schema.entity(Record::reference()).aspect(Record::reference(), Facts::reference())
            .scalar_aspect_field(Record::reference(), Value::reference()) }
    }
}
worth_query_declaration::worth_query_entity!(Record for ScalarSchema);
worth_query_declaration::worth_query_aspect!(Facts for ScalarSchema, Record; identity = AspectIdentity(0x971001), revision = AspectContractRevision(1));
worth_query_declaration::worth_query_field!(Value for ScalarSchema, Record, Facts: u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_declaration::worth_query_field!(Sibling for ScalarSchema, Record, Facts: u64 => U64ApplicationValueBinding, read_write, no_equality);
worth_query_declaration::worth_query_field!(Maybe for ScalarSchema, Record, Facts: optional u64 => U64ApplicationValueBinding, read_write, no_equality);

fn base() -> ApplicationSchemaDeclarationBuilder<ScalarSchema> {
    ApplicationSchemaDeclarationBuilder::for_schema()
        .entity(Record::reference())
        .aspect(Record::reference(), Facts::reference())
}
#[test]
fn explicit_scalar_storage_preserves_binding_and_refuses_extra_or_optional_children() {
    let scalar = ScalarSchema::declaration().unwrap();
    let records = compile_portable_native_contract_records(scalar.erased()).unwrap();
    assert_eq!(records.len(), 1);
    assert!(matches!(
        records[0].contract().shape(),
        worth_foundational::facade::AspectShape::Scalar(ScalarAspectType::UInt64)
    ));
    assert!(
        matches!(records[0].binding(), AspectBinding::EntityField { field } if field.as_str() == "Value")
    );
    let structure = base()
        .field(Record::reference(), Value::reference())
        .build()
        .unwrap();
    let structs = compile_portable_native_contract_records(structure.erased()).unwrap();
    assert!(matches!(
        structs[0].contract().shape(),
        worth_foundational::facade::AspectShape::Struct(_)
    ));
    assert_ne!(scalar.erased().identity(), structure.erased().identity());
    let scalar_meaning = ApplicationAspectValueMeaning::from_members(scalar.erased().members())
        .unwrap()
        .remove(0);
    let struct_meaning = ApplicationAspectValueMeaning::from_members(structure.erased().members())
        .unwrap()
        .remove(0);
    assert_eq!(scalar_meaning.scalar_field(), Some("Value"));
    assert_eq!(
        scalar_meaning.compare_predecessor(&struct_meaning),
        Err(ApplicationAspectMeaningPredecessorDenial::MeaningChangedWithoutRevision)
    );
    installed_read_scope::assert_declared_scopes();
    for invalid in [
        base()
            .scalar_aspect_field(Record::reference(), Value::reference())
            .field(Record::reference(), Sibling::reference())
            .build()
            .unwrap(),
        base()
            .scalar_aspect_field(Record::reference(), Maybe::reference())
            .build()
            .unwrap(),
    ] {
        assert_eq!(
            compile_portable_native_contract_records(invalid.erased())
                .unwrap_err()
                .kind(),
            DenialKind::InvalidAspectShape
        );
    }
}
