pub(crate) use worth_query_testkit_certification_canonical::facade::certification::application_graph::canonical_value::*;

#[test]
fn nondefault_nested_value_uses_derived_binding_catalog_and_native_checkpoint() {
    let authored = authored_value();
    assert_eq!(
        NestedBinding::encode(&authored).unwrap(),
        AspectValue::String(AUTHORED_WIRE.into())
    );
    assert_eq!(
        NestedBinding::decode(&AspectValue::String(AUTHORED_WIRE.into())).unwrap(),
        authored
    );

    let dir = worth_query_testkit_document_retention::facade::directory_home::unique_directory(
        "canonical-value",
    );
    let application = host::open_directory(dir, authored.clone())
        .expect("typed source seed encodes through its binding");
    assert_installed_contract(&application);
    assert_eq!(host::read(&application), authored);
    let dir = application.checkpoint_and_close_for_reopen();

    let restored = host::open_directory(dir, authored.clone())
        .expect("Directory Home reopens with current-format native checkpoint");
    assert_installed_contract(&restored);
    assert_eq!(host::read(&restored), authored);
}

#[test]
fn ordinary_nested_storage_accepts_values_beyond_former_representation_ceilings() {
    let mut authored = authored_value();
    authored.children[0].tag.0 = "λ".repeat(257);
    authored.children = vec![authored.children[0].clone(); 17];
    let encoded = NestedBinding::encode(&authored).unwrap();
    assert_eq!(NestedBinding::decode(&encoded).unwrap(), authored);
    let application = host::install(authored.clone())
        .expect("ordinary canonical storage requires no guessed representation limits");
    assert_eq!(host::read(&application), authored);
}

#[test]
fn unrestricted_binding_still_refuses_a_field_outside_its_declared_grammar() {
    let malformed = AUTHORED_WIRE.replacen("\"title\":", "\"unexpected\":", 1);
    assert!(matches!(
        NestedBinding::decode(&AspectValue::String(malformed.into())),
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::Malformed,
            ..
        })
    ));
}
