//! The actual Native contract compiler consumes the SDK's flat scalar fields.
use super::*;
use worth_query_testkit_decl::fragment_fixture::{declare, Fixture};

#[test]
fn native_contract_compilation_retains_arrays_and_nested_records_as_scalar_leaves() {
    let schema = declare(Fixture).unwrap();
    let records = compile_portable_native_contract_records(schema.erased()).unwrap();
    let metrics = records
        .iter()
        .find(|record| record.entity() == "item" && record.aspect().as_str() == "metrics")
        .unwrap();
    let worth_foundational::facade::AspectShape::Struct(shape) = metrics.contract().shape() else {
        panic!("records must remain ordinary scalar-field structs")
    };
    let names: Vec<_> = shape
        .fields()
        .iter()
        .map(|field| field.key().as_str())
        .collect();
    assert_eq!(
        names,
        [
            "components[0]",
            "components[1]",
            "components[2]",
            "interval.end",
            "interval.start"
        ]
    );
    assert!(shape
        .fields()
        .iter()
        .all(|field| field.value_type() == ScalarAspectType::UInt64));
}
