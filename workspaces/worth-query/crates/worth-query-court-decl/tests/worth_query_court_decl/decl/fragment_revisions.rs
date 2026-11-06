use worth_query_testkit_decl::fragment_revision_fixture::*;
#[test]
fn a_portable_shape_revision_does_not_depend_on_rust_binding_set_or_row_placement() {
    let source = Source::declaration().unwrap();
    let target = Target::declaration().unwrap();
    assert_ne!(
        source.record_bindings()[0].binding_set(),
        target.record_bindings()[0].binding_set()
    );
    assert_ne!(
        source.record_bindings()[0].value_type(),
        target.record_bindings()[0].value_type()
    );
    assert_eq!(source.erased().members(), target.erased().members());
    assert_eq!(source.erased().identity(), target.erased().identity());
}
