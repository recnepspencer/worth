pub(crate) use worth_query_testkit_installation::facade::installation::installation_contracts::application_value_binding_installation::*;

#[test]
fn installed_schema_retains_exact_typed_scalar_codec_and_contract() {
    let installed = installed_index()
        .bind_application_schema(BindingSchema::declaration().unwrap())
        .unwrap();
    let binding = installed
        .value_bindings()
        .field("Component", "Measurement", "Count")
        .unwrap();

    assert_eq!(installed.value_bindings().len(), 1);
    assert_eq!(binding.schema(), &installed.binding_identity());
    assert_eq!(binding.identity().as_str(), "worth.rust.u64");
    assert_eq!(binding.scalar_family(), ScalarAspectType::UInt64);
    assert!(binding.is_readable());
    assert!(binding.admits_native_input());
    let decode = binding
        .native_input_resources(&AspectValue::UInt64(7))
        .unwrap();
    assert_eq!(decode.work_units(), 1);
    assert_eq!(decode.peak_memory_bytes(), 8);
    assert!(binding
        .native_input_resources(&AspectValue::Bool(true))
        .is_err());
    assert!(binding.is_identity());
    assert_eq!(binding.encode(&42_u64).unwrap(), AspectValue::UInt64(42));
    assert_eq!(binding.decode::<u64>(&AspectValue::UInt64(7)).unwrap(), 7);

    let denial = binding.encode(&"wrong Rust type").unwrap_err();
    assert!(matches!(
        denial,
        ApplicationValueEncodeDenial::Validation(_)
    ));
    assert_eq!(denial.binding_identity().as_str(), "worth.rust.u64");
}
