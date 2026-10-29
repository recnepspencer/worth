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
fn registered_foreign_runtime_construction_in_source_tests_is_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] mod tests { #[test] fn crosses_owner() { let _ = foreign::Runtime::new(); } }",
    );
    f.refuses("new source court `tests::crosses_owner` constructs `foreign::Runtime::new`");
}
#[test]
fn imported_aliases_and_local_fixture_calls_cannot_hide_construction() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::Runtime as Renamed; fn fixture() { let _ = Renamed::new(); } #[cfg(test)] mod tests { #[test] fn through_fixture() { super::fixture(); } }");
    f.refuses("through_fixture` constructs `foreign::Runtime::new`");
}
#[test]
fn macro_wrapped_source_test_construction_is_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "wrapped! { #[test] fn generated() { let _ = foreign::Runtime::new(); } }",
    );
    f.refuses("generated` constructs `foreign::Runtime::new`");
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
        "#[test] fn court() { let _ = foreign::Runtime::new(); }",
    );
    let report = source_report(&f.0).unwrap();
    f.write(
        source_baseline::BASELINE,
        &serde_json::to_string_pretty(&report).unwrap(),
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
    f.write(
        "governed/src/lib.rs",
        "#[test] fn court() { let _ = foreign::Runtime::new(); let _ = foreign::OtherRuntime::new(); }",
    );
    f.refuses("foreign::OtherRuntime::new");
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
    f.write("governed/src/lib.rs", "fn fixture() {} mod unrelated { fn fixture() { let _ = foreign::Runtime::new(); } } #[test] fn local_logic() { fixture(); }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn an_existing_fixture_mount_cannot_hide_a_new_test() {
    let f = product();
    f.write("governed/src/lib.rs", "#[cfg(test)] #[path=\"../../sample-testkit/src/world.rs\"] mod setup; #[test] fn old_case() { setup::make(); }");
    f.write(
        "sample-testkit/src/world.rs",
        "pub fn make() { let _ = foreign::Runtime::new(); }",
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
    f.refuses("new_case` constructs `foreign::Runtime::new`");
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
        "pub fn make() { let _ = foreign::Runtime::new(); }",
    );
    f.refuses("case` constructs `foreign::Runtime::new`");
}
#[test]
fn associated_fixture_constructors_are_followed() {
    let f = product();
    f.write("governed/src/lib.rs", "struct Fixture; impl Fixture { fn new()->Self { let _ = foreign::Runtime::new(); Self } } #[test] fn case() { let _ = Fixture::new(); }");
    f.refuses("case` constructs `foreign::Runtime::new`");
}
#[test]
fn foreign_globs_and_local_helper_aliases_are_resolved() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::*; fn make() { let _ = Runtime::new(); } mod tests { use super::make as fixture; #[test] fn case() { fixture(); } }");
    f.refuses("tests::case` constructs `foreign::<glob>::Runtime::new`");
}
#[test]
fn facade_globs_and_type_aliases_preserve_foreign_ownership() {
    let f = product();
    f.write("governed/src/lib.rs", "mod facade { pub use foreign::Runtime; } use facade::*; type Alias=Runtime; #[test] fn case() { let _ = Alias::new(); }");
    f.refuses("case` constructs `foreign::Runtime::new`");
}
#[test]
fn owned_type_shadowing_and_primitive_predicates_are_not_foreign_construction() {
    let f = product();
    f.write("governed/src/lib.rs", "use foreign::Runtime; mod child { struct Runtime; impl Runtime { fn new()->Self {Self} } #[test] fn local_logic() { let _=Runtime::new(); assert!(foreign::is_valid(1)); } }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}

#[test]
fn vocabulary_enum_values_remain_private_unit_logic() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[test] fn case() { let _=foreign::State::Ready; }",
    );
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn value_only_generators_do_not_create_runtime_courts() {
    let f = product();
    f.write("governed/src/lib.rs", "#[cfg(test)] mod tests { resource_contract!(existing_case); resource_contract!(new_case); }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn foreign_vocabulary_constructors_are_valid_private_unit_tests() {
    let f = product();
    f.write("governed/src/lib.rs", "#[test] fn private_logic() { let _=foreign::FieldKey::new(); let _=foreign::AspectKey::new(); let _=foreign::ScalarAspectType::String; }");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn owned_runtime_struct_literals_are_refused() {
    let f = product();
    f.write("governed/src/lib.rs", "struct ApplicationRuntime { generation:u64 } #[test] fn root() { let _=ApplicationRuntime { generation:0 }; }");
    f.refuses("ApplicationRuntime");
}
#[test]
fn runtime_factory_builds_are_refused_without_registering_the_builder() {
    let f = product();
    f.write("governed/src/lib.rs", "struct Builder; impl Builder { fn new()->Self{Self} fn build(self)->foreign::Runtime {foreign::Runtime::new()} } #[test] fn configuration(){let _=Builder::new();} #[test] fn root(){Builder::new().build();}");
    let messages = f.messages();
    assert!(
        messages
            .iter()
            .any(|m| m.contains("root` constructs `foreign::Runtime::<factory>")),
        "{messages:?}"
    );
    assert!(
        !messages.iter().any(|m| m.contains("configuration`")),
        "{messages:?}"
    );
}
#[test]
fn runtime_arguments_to_macro_families_are_refused() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[cfg(test)] mod tests { generated_cases!(case,foreign::Runtime::new()); }",
    );
    f.refuses("foreign::Runtime::new");
}

#[test]
fn registered_async_runtime_harnesses_are_source_courts() {
    let f = product();
    f.append(
        super::super::runtime_roots::CONFIG,
        "[[roots]]\ntype='tokio::runtime::Runtime'\nreason='Owns the asynchronous executor'\n",
    );
    f.write(
        "governed/src/lib.rs",
        "#[tokio::test] async fn runtime_harness() { assert_eq!(1,1); }",
    );
    f.refuses("tokio::runtime::Runtime::<test-harness>");
}

#[test]
fn compact_baselines_refuse_duplicate_test_entries() {
    let f = product();
    f.write(
        "governed/src/lib.rs",
        "#[test] fn root(){let _=foreign::Runtime::new();}",
    );
    let report = source_report(&f.0).unwrap();
    let entries = report.as_array().unwrap();
    let duplicated = serde_json::json!([entries[0], entries[0]]);
    f.write(
        source_baseline::BASELINE,
        &serde_json::to_string(&duplicated).unwrap(),
    );
    f.refuses("duplicate test entry");
}
#[test]
fn testkit_names_do_not_require_the_out_of_scope_role_registry() {
    let f = product();
    f.write(
        "governed/Cargo.toml",
        "[package]\nname='sample-testkit-identity'\n[lib]\ntest=false\n",
    );
    f.write("governed/src/lib.rs", "pub fn identity()->u64{1}");
    assert!(f.messages().is_empty(), "{:?}", f.messages());
}
#[test]
fn misnamed_testkits_are_refused() {
    let f = product();
    f.write(
        "governed/Cargo.toml",
        "[package]\nname='sample-testkit-Invalid'\n[lib]\ntest=false\n",
    );
    f.write("governed/src/lib.rs", "pub fn identity()->u64{1}");
    f.refuses("misnamed testkit");
}
