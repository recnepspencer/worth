use super::{
    super::{source_baseline, source_report},
    fixture::Fixture,
};

fn product() -> Fixture {
    let f = Fixture::new();
    f.write(
        "governed/Cargo.toml",
        "[package]\nname='product'\n[dependencies]\nforeign={path='../foreign'}\n",
    );
    std::fs::remove_dir_all(f.0.join("governed/tests")).unwrap();
    f
}
#[test]
fn new_foreign_construction_in_source_tests_is_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] mod tests { #[test] fn crosses_owner() { let _ = foreign::Value::new(); } }",
    );
    f.refuses("new source court `tests::crosses_owner` constructs `foreign::Value::new`");
}
#[test]
fn imported_aliases_and_local_fixture_calls_cannot_hide_construction() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::Value as Renamed; fn fixture() { let _ = Renamed::new(); } #[cfg(test)] mod tests { #[test] fn through_fixture() { super::fixture(); } }");
    f.refuses("through_fixture` constructs `foreign::Value::new`");
}
#[test]
fn macro_wrapped_source_test_construction_is_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "wrapped! { #[test] fn generated() { let _ = foreign::Value::new(); } }",
    );
    f.refuses("generated` constructs `foreign::Value::new`");
}
#[test]
fn source_runtime_construction_does_not_need_an_external_import() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[test] fn runtime() { let _ = ApplicationRuntime::new(); }",
    );
    f.refuses("runtime` constructs `ApplicationRuntime::new`");
}
#[test]
fn existing_evidence_is_reported_but_new_evidence_in_the_same_test_is_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[test] fn court() { let _ = foreign::Value::new(); }",
    );
    let report = source_report(&f.0).unwrap();
    f.write(
        source_baseline::BASELINE,
        &serde_json::to_string_pretty(&report).unwrap(),
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
    f.write(
        "governed/src/lib.rs",
        "#[test] fn court() { let _ = foreign::Value::new(); let _ = foreign::Other::new(); }",
    );
    f.refuses("foreign::Other::new");
}
#[test]
fn private_logic_and_foreign_type_references_are_not_construction() {
    let f = product();
    f.write("governed/src/lib.rs", "struct Local; impl Local { fn new() -> Self { Self } } #[test] fn local_logic() { let _ = Local::new(); let _: Option<foreign::Value> = None; let _ = std::vec::Vec::<u8>::new(); }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn product_source_cannot_mount_an_integration_test_module() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] #[path=\"../tests/court.rs\"] mod hidden;",
    );
    f.write("governed/tests/court.rs", "#[test] fn hidden() {} ");
    f.refuses("cannot mount a tests/ module");
}

#[test]
fn private_source_test_modules_may_live_in_src_tests_directories() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] #[path=\"tests/private.rs\"] mod tests;",
    );
    f.write(
        "governed/src/tests/private.rs",
        "#[test] fn local() { assert_eq!(2+2,4); }",
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn same_named_fixtures_in_other_modules_do_not_create_false_violations() {
    let f = product();
    f.write("governed/src/lib.rs", "fn fixture() {} mod unrelated { fn fixture() { let _ = foreign::Value::new(); } } #[test] fn local_logic() { fixture(); }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn an_existing_fixture_mount_cannot_hide_a_new_test() {
    let f = product();
    f.write("governed/src/lib.rs", "#[cfg(test)] #[path=\"../../sample-testkit/src/world.rs\"] mod setup; #[test] fn old_case() { setup::make(); }");
    f.write(
        "sample-testkit/src/world.rs",
        "pub fn make() { let _ = foreign::Value::new(); }",
    );
    f.write(
        source_baseline::BASELINE,
        &serde_json::to_string_pretty(&source_report(&f.0).unwrap()).unwrap(),
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
    f.append(
        "governed/src/lib.rs",
        "#[test] fn new_case() { setup::make(); }",
    );
    f.refuses("new_case` constructs `foreign::Value::new`");
}
#[test]
fn path_aliases_keep_their_rust_module_identity() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[path=\"world_fixture.rs\"] mod setup; #[test] fn case() { setup::make(); }",
    );
    f.write(
        "governed/src/world_fixture.rs",
        "pub fn make() { let _ = foreign::Value::new(); }",
    );
    f.refuses("case` constructs `foreign::Value::new`");
}
#[test]
fn associated_fixture_constructors_are_followed() {
    let f = product();
    f.write("governed/src/lib.rs", "struct Fixture; impl Fixture { fn new()->Self { let _ = foreign::Value::new(); Self } } #[test] fn case() { let _ = Fixture::new(); }");
    f.refuses("case` constructs `foreign::Value::new`");
}
#[test]
fn foreign_globs_and_local_helper_aliases_are_resolved() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::*; fn make() { let _ = Value::new(); } mod tests { use super::make as fixture; #[test] fn case() { fixture(); } }");
    f.refuses("tests::case` constructs `foreign::<glob>::Value::new`");
}
#[test]
fn facade_globs_and_type_aliases_preserve_foreign_ownership() {
    let f = product();
    f.write("governed/src/lib.rs", "mod facade { pub use foreign::Value; } use facade::*; type Alias=Value; #[test] fn case() { let _ = Alias::new(); }");
    f.refuses("case` constructs `foreign::Value::new`");
}
#[test]
fn owned_type_shadowing_and_primitive_predicates_are_not_foreign_construction() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::Value; mod child { struct Value; impl Value { fn new()->Self {Self} } #[test] fn local_logic() { let _=Value::new(); assert!(foreign::is_valid(1)); } }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn unit_enum_variants_are_foreign_value_construction() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[test] fn case() { let _=foreign::State::Ready; }",
    );
    f.refuses("case` constructs `foreign::State::Ready`");
}
#[test]
fn opaque_generators_cannot_add_invisible_source_tests() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] mod tests { resource_contract!(existing_case); }",
    );
    f.write(
        source_baseline::BASELINE,
        &serde_json::to_string_pretty(&source_report(&f.0).unwrap()).unwrap(),
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
    f.write("governed/src/lib.rs", "#[cfg(test)] mod tests { resource_contract!(existing_case); resource_contract!(new_case); }");
    f.refuses("opaque source-test generator resource_contract(new_case)");
}
